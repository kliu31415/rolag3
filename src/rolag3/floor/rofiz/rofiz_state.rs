use std::{rc::{Rc, Weak}, cell::RefCell};

use crate::{rolag3::floor::{room_object::room_object_def::RoomObjectId, rofiz::rofiz_object::RofizObjectMovement}, geometry::shape::{Shape, BoundingBox}};

use super::rofiz_object::{RofizObjBasicWall, RofizObjMovable, Hitbox, Transformation};


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

    wall_x_end: usize,
    wall_y_end: usize,
    has_wall_at_coordinate: Vec<Vec<Option<Rc<RefCell<RofizObjBasicWall>>>>>,

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
            wall_x_end: 0,
            wall_y_end: 0,
            has_wall_at_coordinate: Vec::new(),
            obj_creation_counter: 0,
        }
    }

    pub fn finalize_start_floor(&mut self) {
        assert!(!self.floor_started, "cannot finalize floor twice");

        self.floor_started = true;

        let mut max_x = 0;
        let mut max_y = 0;
        for bw in self.basic_walls.iter().map(|x| x.borrow()) {
            max_x = u32::max(bw.x, max_x);
            max_y = u32::max(bw.y, max_y);
        }

        if max_x > Self::MAX_XY_WARN || max_y > Self::MAX_XY_WARN {
            log::warn!("basic wall max_x({}) and max_y({}) are large. This has negative performance implications", max_x, max_y);
        }

        self.wall_x_end = (max_x + 1) as usize;
        self.wall_y_end = (max_y + 1) as usize;
        let mut has_wall_at_coordinate = vec![vec![None; (max_y+1) as usize]; (max_x+1) as usize];
        for bw in self.basic_walls.iter().map(|x| x) {
            has_wall_at_coordinate[bw.borrow().x as usize][bw.borrow().y as usize] = Some(bw.clone());
        }

        self.has_wall_at_coordinate = has_wall_at_coordinate;
    }

    pub fn start_new_tick(&mut self) {
        assert!(self.floor_started, "floor must be started before Rofiz starts new tick");
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
                panic!("Basic wall with id {} has Rc={}. Basic walls can't be deleted after room finalization, so expected Rc>1.", x.borrow().id, count);
            }
        });
    }

    pub fn remove_wall_at(&mut self, x: u32, y: u32) {
        assert!(!self.floor_started, "cannot remove basic wall after Rofiz floor started");
        let old_len = self.basic_walls.len();
        self.basic_walls.retain(|bw| x!=bw.borrow().x || y!=bw.borrow().y);
        let new_len = self.basic_walls.len();
        assert!(new_len+1 == old_len, "expected to remove one Rofiz wall at (x, y) = ({}, {}). old_len={}, new_len={}", x, y, old_len, new_len);
    }

    pub fn add_basic_wall(&mut self, floor_object_id: RoomObjectId, x: u32, y: u32) -> RofizObjectRef {
        assert!(!self.floor_started, "cannot add basic wall after Rofiz floor started");
        let new_wall = Self::new_rofiz_obj_basic_wall(self, floor_object_id, x, y);
        let rc = Rc::new(RefCell::new(new_wall));
        self.basic_walls.push(rc.clone());
        RofizObjectRef::new(RofizObjectRefVal::BasicWall(Rc::downgrade(&rc)))
    }

    pub fn add_nonspectral_unit(&mut self, floor_object_id: RoomObjectId, hitbox: Hitbox) -> RofizObjectRef {
        // todo: add logic here to verify that the nonspectral unit doesn't intersect with any other nonspectral unit
        // or any wall.
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
            RofizObjectRefVal::BasicWall(_) => panic!("accessing BasicWall in move_object() is not supported"),
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

    pub fn get_movable_object_xformed_shape(&self, obj_ref: &RofizObjectRef) -> Shape {
        let wp = match &obj_ref.val {
            RofizObjectRefVal::BasicWall(_) => panic!("accessing BasicWall in get_movable_object_xform() is not supported"),
            RofizObjectRefVal::BasicProjectile(p) => p.clone(),
            RofizObjectRefVal::_SpectralUnit(p) => p.clone(),
            RofizObjectRefVal::NonspectralUnit(p) => p.clone(),
        };
        let rc = wp.upgrade().unwrap();
        let hitbox = &rc.borrow().current;
        hitbox.transformation.get_transformed_shape(&hitbox.shape)
    }
    
    pub fn move_objects_and_find_collisions(&mut self) -> Vec<RofizCollision> {
        assert!(self.floor_started, "floor must be started before Rofiz moves objects and finds collisions");

        self.basic_projectiles.retain(|x| !matches!(x.borrow().movement, RofizObjectMovement::_Delete()));
        self.nonspectral_units.retain(|x| !matches!(x.borrow().movement, RofizObjectMovement::_Delete()));
        self.spectral_units.retain(|x| !matches!(x.borrow().movement, RofizObjectMovement::_Delete()));
        for obj_rc in self.movable_objs_iter() {
            let mut obj = obj_rc.as_ref().borrow_mut();
            obj.initial_hitbox = obj.current.transformation.get_transformed_shape(&obj.current.shape);
            obj.temp_hitbox = match obj.movement {
                RofizObjectMovement::NoMove() => obj.current.transformation.get_transformed_shape(&obj.current.shape),
                RofizObjectMovement::Move(ref t) => (obj.current.transformation.add(t)).get_transformed_shape(&obj.current.shape),
                RofizObjectMovement::MoveWithFallbacks(ref v) => (obj.current.transformation.add(&v[0])).get_transformed_shape(&obj.current.shape),
                RofizObjectMovement::NewHitbox(ref h) => h.transformation.get_transformed_shape(&h.shape), 
                RofizObjectMovement::_Delete() => panic!("there should be no rofiz objects with Delete movement. Loc 1a."),
            };
            obj.bounding_box = match obj.movement {
                RofizObjectMovement::NoMove() => BoundingBox::of_shape(&obj.temp_hitbox),
                RofizObjectMovement::Move(_) => {
                    let mut b = BoundingBox::of_shape(&obj.initial_hitbox);
                    b.combine(&BoundingBox::of_shape(&obj.temp_hitbox));
                    b
                }
                RofizObjectMovement::MoveWithFallbacks(ref v) => {
                    let mut all_bb = BoundingBox::of_shape(&obj.initial_hitbox);
                    for t in v {
                        let bb = BoundingBox::of_shape(&obj.current.transformation.add(t).get_transformed_shape(&obj.current.shape));
                        all_bb.combine(&bb);
                    }
                    all_bb
                }
                RofizObjectMovement::NewHitbox(_) => {
                    let mut b = BoundingBox::of_shape(&obj.initial_hitbox);
                    b.combine(&BoundingBox::of_shape(&obj.temp_hitbox));
                    b
                }
                RofizObjectMovement::_Delete() => panic!("there should be no rofiz objects with Delete movement. Loc 1b."),
            };
            obj.fallback_idx = 0;
            obj.move_successful = true;
        }

        let mut nsu_bb_overlap = Vec::new();
        for i in 0..self.nonspectral_units.len() {
            let mut i_bb_overlap = Vec::new();
            for j in 0..i {
                let nsu_i = self.nonspectral_units[i].borrow();
                let nsu_j = self.nonspectral_units[j].borrow();
                if BoundingBox::overlap(&nsu_i.bounding_box, &nsu_j.bounding_box) {
                    i_bb_overlap.push(j);
                }
            }
            nsu_bb_overlap.push(i_bb_overlap);
        }

        let mut collisions = Vec::new();

        // Phase 1: Check for collisions between nonspectral units and basic walls
        // At any point, the invariant that nsus[0..i] are in valid positions should hold, i.e. none of them overlap.
        let mut i = 0;

        while i < self.nonspectral_units.len() {
            // verify that nsu[i] doesn't overlap with any walls
            let mut nsu_i = self.nonspectral_units[i].as_ref().borrow_mut();
            // [start, end). Note that half-open interval
            let xstart = usize::clamp(nsu_i.bounding_box.x1 as usize, 0, self.wall_x_end);
            let xend = usize::clamp(nsu_i.bounding_box.x2 as usize + 1, 0, self.wall_x_end);
            let ystart = usize::clamp(nsu_i.bounding_box.y1 as usize, 0, self.wall_y_end);
            let yend = usize::clamp(nsu_i.bounding_box.y2 as usize + 1, 0, self.wall_y_end);
            for x in xstart..xend {
                for y in ystart..yend {
                    if let Some(ref bw) = self.has_wall_at_coordinate[x][y] {
                        let bw = bw.borrow();
                        if nsu_i.overlaps_ro_wall(&bw) {
                            collisions.push(RofizCollision::new(nsu_i.room_object_id, bw.id));
                            // keep moving the unit back while both of the following hold:
                            // 1. the unit is moved back to a different position
                            // 2. the different position overlaps with a wall.
                            // Remember, we assert that the unit's original position must never overlap with a basic wall
                            while Self::move_back(&mut nsu_i) && nsu_i.overlaps_ro_wall(&bw) {}
                            // since the unit moved, it needs to be rechecked against all walls.
                        }
                    }
                }
            }

            let mut i_override = None;
            let mut bbo_idx = 0;
            while bbo_idx < nsu_bb_overlap[i].len() {
                let j = nsu_bb_overlap[i][bbo_idx];
                if j > i {
                    break;
                }
                bbo_idx += 1;

                let mut nsu_j = self.nonspectral_units[j].as_ref().borrow_mut();
                if nsu_i.overlaps_ro_movable(&nsu_j) {
                    collisions.push(RofizCollision::new(nsu_i.room_object_id, nsu_j.room_object_id));

                    // if this collision can be solved by only one of nsu_i and one of nsu_j moving back, do that.
                    // This is to avoid deadlock, e.g. object A is right above object B. Object A's velocity is
                    // (2, 1) and object B's velocity is (2, 2). In this case, object A should ideally move fully, and
                    // object B might be able to move to a fallback location (rather than stay in its original place).

                    // check if this collision can be solved just by moving nsu_i back. If so, only move nsu_i back.
                    // nsu_i will need to be rechecked against all walls.
                    if !nsu_i.initial_overlaps_ro_movable(&nsu_j) {
                        while Self::move_back(&mut nsu_i) && nsu_i.overlaps_ro_movable(&nsu_j) {}
                        i_override = Some(i);
                        break;
                    }

                    // check if this collision can be solved just by moving nsu_j back. If so, only move nsu_j back.
                    if !nsu_j.initial_overlaps_ro_movable(&nsu_i) {
                        while Self::move_back(&mut nsu_j) && nsu_i.overlaps_ro_movable(&nsu_j) {}
                        i_override = Some(j);
                        break;
                    }

                    // Otherwise, move both of them back. Note that we can't just move nsu_i back to its original location,
                    // because even though nsu_i's original location intersects with nsu_j's current temp location, one
                    // of nsu_i's intermediate fallback locations might not intersect with nsu_j.
                    while Self::move_back(&mut nsu_i) && nsu_i.overlaps_ro_movable(&nsu_j) {}

                    if nsu_i.overlaps_ro_movable(&nsu_j) {
                        while Self::move_back(&mut nsu_j) && nsu_i.overlaps_ro_movable(&nsu_j) {}
                        // nsus[0..j] are still valid, but nsus[j..i] aren't necessarily, because j was moved.
                        i_override = Some(j);
                        break;
                    } else {
                        // moving nsu_i back is enough to resolve the collision, and nsus[0..i] are still 
                        // in valid final positions. However, since nsu_i moved, it needs to be rechecked with all
                        // other walls and nsu_js
                        i_override = Some(i);
                        break;
                    }
                }
            }

            i = match i_override {
                Some(v) => v,
                None => i + 1,
            }
        }

        // Phase 2: Spectral units (TODO)

        // Phase 3: Projectiles
        for bp_rc in self.basic_projectiles.iter() {
            let bp: std::cell::RefMut<'_, RofizObjMovable> = bp_rc.as_ref().borrow_mut();

            // [start, end). Note that half-open interval
            let xstart = usize::clamp(bp.bounding_box.x1 as usize, 0, self.wall_x_end);
            let xend = usize::clamp(bp.bounding_box.x2 as usize + 1, 0, self.wall_x_end);
            let ystart = usize::clamp(bp.bounding_box.y1 as usize, 0, self.wall_y_end);
            let yend = usize::clamp(bp.bounding_box.y2 as usize + 1, 0, self.wall_y_end);
            for x in xstart..xend {
                for y in ystart..yend {
                    if let Some(ref bw) = self.has_wall_at_coordinate[x][y] {
                        let bw = bw.as_ref().borrow();
                        if bp.overlaps_ro_wall(&bw) {
                            collisions.push(RofizCollision::new(bp.room_object_id, bw.room_object_id));
                        }
                    }
                }
            }

            for nsu_rc in self.nonspectral_units.iter() {
                let nsu = nsu_rc.as_ref().borrow_mut();
                if bp.overlaps_ro_movable(&nsu) {
                    collisions.push(RofizCollision::new(bp.room_object_id, nsu.room_object_id));
                }
            }
        }

        // wrap up by officially moving objects
        for mo_rc in self.movable_objs_iter() {
            let mut mo = mo_rc.as_ref().borrow_mut();
            if let RofizObjectMovement::MoveWithFallbacks(ref v) = mo.movement {
                if mo.fallback_idx < v.len() {
                    mo.current.transformation = mo.current.transformation.add(&v[mo.fallback_idx]);
                }
            } else if mo.move_successful {
                match mo.movement {
                    RofizObjectMovement::NoMove() => {},
                    RofizObjectMovement::Move(ref t) => mo.current.transformation = mo.current.transformation.add(t),
                    RofizObjectMovement::MoveWithFallbacks(_) => panic!("Rofiz MoveWithFallbacks should not be hit here"),
                    RofizObjectMovement::NewHitbox(ref h) => mo.current = h.clone(),
                    RofizObjectMovement::_Delete() => panic!("there should be no rofiz objects with Delete movement. Loc 2."),
                }
            }
        }
        collisions
    }

    // returns true if the object was moved back to a different location
    fn move_back(rom: &mut RofizObjMovable) -> bool {
        match rom.movement {
        RofizObjectMovement::MoveWithFallbacks(ref v) => {
            if rom.fallback_idx < v.len() {
                rom.fallback_idx += 1;
                if rom.fallback_idx < v.len() {
                    let new_xform = rom.current.transformation.add(&v[rom.fallback_idx]); 
                    rom.temp_hitbox = new_xform.get_transformed_shape(&rom.current.shape);
                } else {
                    rom.temp_hitbox = rom.current.transformation.get_transformed_shape(&rom.current.shape);
                }
                true
            } else {
                false
            }
        }
        _ => {
            if rom.move_successful {
                rom.move_successful = false;
                rom.temp_hitbox = rom.current.transformation.get_transformed_shape(&rom.current.shape);
                true
            } else {
                false
            }
        }
        }
    }

    fn movable_objs_iter(&mut self) -> impl Iterator<Item = &Rc<RefCell<RofizObjMovable>>> {
        self.basic_projectiles.iter()
                .chain(self.spectral_units.iter())
                .chain(self.nonspectral_units.iter())
    }

    fn new_rofiz_obj_basic_wall(&mut self, floor_object_id: RoomObjectId, x: u32, y: u32) -> RofizObjBasicWall {
        self.obj_creation_counter += 1;
        let shape = Shape::of_square(x as f32, y as f32, 1.0);
        let bounding_box = BoundingBox::of_shape(&shape);
        RofizObjBasicWall { 
            id: self.obj_creation_counter, 
            external_ref_count: Rc::new(()),
            room_object_id: floor_object_id,
            x, 
            y, 
            shape,
            bounding_box,
        }
    }

    fn new_rofiz_obj_movable(&mut self, hitbox: Hitbox, floor_object_id: RoomObjectId) -> RofizObjMovable {
        self.obj_creation_counter += 1;
        RofizObjMovable { 
            id: self.obj_creation_counter, 
            external_ref_count: Rc::new(()),
            current: hitbox, 
            movement: RofizObjectMovement::NoMove(), 
            move_with_fallbacks_idx: 0,
            bounding_box: BoundingBox::zero_state(),
            temp_hitbox: Shape::dummy(),
            initial_hitbox: Shape::dummy(),
            move_successful: false, // dummy
            fallback_idx: 0, // dummy
            room_object_id: floor_object_id,
        }
    }
}

pub struct RofizCollision {
    pub room_obj_id1: RoomObjectId,
    pub room_obj_id2: RoomObjectId,
}

impl RofizCollision {
    fn new(room_obj_id1: RoomObjectId, room_obj_id2: RoomObjectId) -> Self {
        Self {
            room_obj_id1,
            room_obj_id2,
        }
    }
}
