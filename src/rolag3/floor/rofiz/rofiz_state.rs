use std::{sync::{Arc, atomic::AtomicU32}, ops::Range};

use crate::{rolag3::floor::{room_object::room_object_def::RoomObjectRef, rofiz::rofiz_object::RofizObjectMovement}, geometry::shape::{Shape, BoundingBox}};

use super::{rofiz_object::{RofizObjBasicWall, RofizObjMovable, Hitbox, Transformation}, object_pool::{RofizObjPoolRef, RofizObjPool}};

#[derive(Debug, Clone)]
pub struct RofizObjectRef {
    _ref_count: Arc<()>,
    pool_ref: RofizObjPoolRef, // purely used to hide the actual ref from external callers
    rofiz_state_id: RofizStateId, // purely used for bug-checking
}

impl RofizObjectRef {
    fn new(_ref_count: Arc<()>, pool_ref: RofizObjPoolRef, rofiz_state_id: RofizStateId) -> Self {
        Self {
            _ref_count,
            pool_ref,
            rofiz_state_id,
        }
    }
}

type RofizStateId = u32;

pub struct RofizState {
    rs_id: RofizStateId,

    // only used before floor start
    floor_started: bool,

    obj_pool: RofizObjPool,
    basic_walls: Vec<RofizObjPoolRef>,
    basic_projectiles: Vec<RofizObjPoolRef>,
    spectral_units: Vec<RofizObjPoolRef>,
    nonspectral_units: Vec<RofizObjPoolRef>,

    wall_x_end: usize,
    wall_y_end: usize,
    has_wall_at_coordinate: Vec<Vec<Option<RofizObjPoolRef>>>,

    obj_creation_counter: usize,

    spatial_grid: Vec<Vec<Vec<usize>>>,
}

impl RofizState {
    const MAX_XY_WARN: u32 = 1000;

    pub fn new() -> Self {
        static ID_COUNTER: AtomicU32 = AtomicU32::new(0);
        let rs_id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if rs_id == u32::MAX {
            log::warn!("Rofiz State ID is u32::MAX. \
                        This won't cause bugs, but it will make catching certain rofiz-related bugs harder");
        }

        Self {
            rs_id,
            floor_started: false,
            obj_pool: RofizObjPool::new(),
            basic_walls: Vec::new(),
            basic_projectiles: Vec::new(),
            spectral_units: Vec::new(),
            nonspectral_units: Vec::new(),
            wall_x_end: 0,
            wall_y_end: 0,
            has_wall_at_coordinate: Vec::new(),
            obj_creation_counter: 0,
            spatial_grid: Vec::new(),
        }
    }

    #[inline(never)]
    pub fn finalize_start_floor(&mut self) {
        assert!(!self.floor_started, "cannot finalize floor twice");

        self.floor_started = true;

        let mut max_x = 0;
        let mut max_y = 0;
        for bw_ref in self.basic_walls.iter() {
            let bw = self.obj_pool.get_bw(bw_ref);
            max_x = u32::max(bw.x, max_x);
            max_y = u32::max(bw.y, max_y);
        }

        if max_x > Self::MAX_XY_WARN || max_y > Self::MAX_XY_WARN {
            log::warn!("basic wall max_x({}) and max_y({}) are large. This has negative performance implications", max_x, max_y);
        }

        self.wall_x_end = (max_x + 1) as usize;
        self.wall_y_end = (max_y + 1) as usize;
        let mut has_wall_at_coordinate = vec![vec![None; self.wall_y_end]; self.wall_x_end];
        for bw_ref in self.basic_walls.iter() {
            let bw = self.obj_pool.get_bw(bw_ref);
            has_wall_at_coordinate[bw.x as usize][bw.y as usize] = Some(*bw_ref);
        }

        self.has_wall_at_coordinate = has_wall_at_coordinate;
        self.spatial_grid = vec![vec![Vec::new(); self.wall_y_end]; self.wall_x_end]
    }

    #[inline(never)]
    pub fn start_new_tick(&mut self, run_validation: bool) {
        assert!(self.floor_started, "floor must be started before Rofiz starts new tick");
        for obj in self.basic_projectiles.iter()
                .chain(self.spectral_units.iter())
                .chain(self.nonspectral_units.iter()) {
            self.obj_pool.get_mo_mut(obj).movement = RofizObjectMovement::NoMove();
        }

        if run_validation {
            self.basic_walls.iter().for_each(|x| {
                let count = Arc::strong_count(&self.obj_pool.get_bw(x).external_ref_count);
                if count <= 1 {
                    panic!("Basic wall with id {} has Rc={}. Basic walls can't be deleted after room finalization, so expected Rc>1.", &self.obj_pool.get_bw(x).id, count);
                }
            });
        }
    }

    pub fn remove_wall_at(&mut self, x: u32, y: u32, expected: Range<usize>) {
        assert!(!self.floor_started, "cannot remove basic wall after Rofiz floor started");
        let old_len = self.basic_walls.len();
        self.basic_walls.retain(|bw_ref| {
            let bw = self.obj_pool.get_bw(&bw_ref);
            if x==bw.x && y==bw.y {
                self.obj_pool.del_bw(bw_ref);
                return false;
            }
            return true;
        });
        let new_len = self.basic_walls.len();
        let diff = old_len - new_len;
        assert!(expected.contains(&diff), "expected to remove {:?} Rofiz wall at (x, y) = ({}, {}). old_len={}, new_len={}", expected, x, y, old_len, new_len);
    }

    pub fn add_basic_wall(&mut self, floor_object_id: RoomObjectRef, x: u32, y: u32) -> RofizObjectRef {
        assert!(!self.floor_started, "cannot add basic wall after Rofiz floor started");
        let new_wall = Self::new_rofiz_obj_basic_wall(self, floor_object_id, x, y);
        let ref_count = new_wall.external_ref_count.clone();
        let pool_ref = self.obj_pool.add_bw(new_wall);
        self.basic_walls.push(pool_ref);
        RofizObjectRef::new(ref_count, pool_ref, self.rs_id)
    }

    pub fn add_nonspectral_unit(&mut self, floor_object_id: RoomObjectRef, hitbox: Hitbox) -> RofizObjectRef {
        // todo: add logic here to verify that the nonspectral unit doesn't intersect with any other nonspectral unit
        // or any wall.
        let obj = self.new_rofiz_obj_movable(hitbox, floor_object_id, false);
        let ref_count = obj.external_ref_count.clone();
        let pool_ref = self.obj_pool.add_mo(obj);
        self.nonspectral_units.push(pool_ref);
        RofizObjectRef::new(ref_count, pool_ref, self.rs_id)
    }

    pub fn add_spectral_unit(&mut self, floor_object_id: RoomObjectRef, hitbox: Hitbox) -> RofizObjectRef {
        let obj = self.new_rofiz_obj_movable(hitbox, floor_object_id, true);
        let ref_count = obj.external_ref_count.clone();
        let pool_ref = self.obj_pool.add_mo(obj);
        self.spectral_units.push(pool_ref);
        RofizObjectRef::new(ref_count, pool_ref, self.rs_id)
    }

    pub fn add_basic_projectile(&mut self, floor_object_id: RoomObjectRef, hitbox: Hitbox) -> RofizObjectRef {
        let obj = self.new_rofiz_obj_movable(hitbox, floor_object_id, true);
        let ref_count = obj.external_ref_count.clone();
        let pool_ref = self.obj_pool.add_mo(obj);
        self.basic_projectiles.push(pool_ref);
        RofizObjectRef::new(ref_count, pool_ref, self.rs_id)
    }

    pub fn steal_movable_object_shape(&mut self, obj_ref: &RofizObjectRef) -> Shape {
        assert_eq!(self.rs_id, obj_ref.rofiz_state_id);
        if obj_ref.pool_ref.is_bw {
            unimplemented!();
        }
        let rom = self.obj_pool.get_mo_mut(&obj_ref.pool_ref);
        let mut stolen = Shape::default();
        std::mem::swap(&mut stolen, &mut rom.cached_mem_hitbox.shape);
        stolen
    }

    pub fn move_object(&mut self, obj_ref: &RofizObjectRef, movement: RofizObjectMovement) {
        assert_eq!(self.rs_id, obj_ref.rofiz_state_id);
        assert!(!obj_ref.pool_ref.is_bw);
        self.obj_pool.get_mo_mut(&obj_ref.pool_ref).movement = movement;
    }

    pub fn get_movable_object_xform(&self, obj_ref: &RofizObjectRef) -> Transformation {
        assert_eq!(self.rs_id, obj_ref.rofiz_state_id);
        assert!(!obj_ref.pool_ref.is_bw);
        self.obj_pool.get_mo(&obj_ref.pool_ref).current.transformation
    }

    pub fn get_movable_object_xformed_shape(&self, obj_ref: &RofizObjectRef) -> Shape {
        assert_eq!(self.rs_id, obj_ref.rofiz_state_id);
        assert!(!obj_ref.pool_ref.is_bw);
        let hitbox = &self.obj_pool.get_mo(&obj_ref.pool_ref).current;
        let mut ret = Shape::dummy();
        hitbox.transformation.replace_shape_with_transformed(&mut ret,&hitbox.shape);
        ret
    }
    
    #[inline(never)]
    pub fn move_objects_and_find_collisions(&mut self) -> Vec<RofizCollision> {
        assert!(self.floor_started, "floor must be started before Rofiz moves objects and finds collisions");
        self.moafc1();
        self.moafc2();
        let nsu_bb_overlap = self.moafc3();

        let mut collisions = Vec::new();

        // Phase 1: Check for collisions between nonspectral units and basic walls
        // At any point, the invariant that nsus[0..i] are in valid positions should hold, i.e. none of them overlap.
        self.moafc4(&mut collisions, nsu_bb_overlap);
        let mut spatial_grid_id_to_obj = self.moafc5();

        // Phase 2 (TODO): Semispectral units?

        // Phase 3: Spectral Units + Projectiles
        self.moafc6(&mut collisions, &mut spatial_grid_id_to_obj);

        // wrap up by officially moving objects
        self.moafc7();

        self.moafc8(&mut collisions);
        collisions
    }

    #[inline(never)]
    fn moafc1(&mut self) {
        // TODO: is Arc::strong_count guaranteed to be up-to-date? For example, if Arc::strong_count is decremented
        // before moafc1(), is the decrement guaranteed to reflect here? 
        // I think it is if the same thread creates, reads, and drops Arcs, but I'm not sure.
        self.basic_projectiles.retain(|x| {
            if matches!(self.obj_pool.get_mo_mut(x).movement, RofizObjectMovement::_Delete()) ||
            Arc::strong_count(&self.obj_pool.get_mo_mut(x).external_ref_count) == 1 {
                self.obj_pool.del_mo(x);
                return false;
            }
            true
        });
        self.nonspectral_units.retain(|x| {
            if matches!(self.obj_pool.get_mo_mut(x).movement, RofizObjectMovement::_Delete()) ||
            Arc::strong_count(&self.obj_pool.get_mo_mut(x).external_ref_count) == 1 {
                self.obj_pool.del_mo(x);
                return false;
            }
            true
        });
        self.spectral_units.retain(|x| {
            if matches!(self.obj_pool.get_mo_mut(x).movement, RofizObjectMovement::_Delete()) ||
            Arc::strong_count(&self.obj_pool.get_mo_mut(x).external_ref_count) == 1 {
                self.obj_pool.del_mo(x);
                return false;
            }
            true
        });
    }

    #[inline(never)]
    fn moafc2(&mut self) {
        self.obj_pool.start_moafc();
    }

    #[inline(never)]
    fn moafc3(&mut self) -> Vec<Vec<usize>> {
        let mut nsu_bb_overlap = Vec::new();
        for i in 0..self.nonspectral_units.len() {
            let mut i_bb_overlap = Vec::new();
            for j in 0..i {
                let (nsu_i, nsu_j) = self.obj_pool.get_mo_mo_mut(&self.nonspectral_units[i], &self.nonspectral_units[j]);
                if BoundingBox::overlap(&nsu_i.bounding_box, &nsu_j.bounding_box) {
                    i_bb_overlap.push(j);
                }
            }
            nsu_bb_overlap.push(i_bb_overlap);
        }
        nsu_bb_overlap
    }

    #[inline(never)]
    fn moafc4(&mut self, collisions: &mut Vec<RofizCollision>, nsu_bb_overlap: Vec<Vec<usize>>) {
        let mut i = 0;
        // asserts on "iterations" is used to ensure that this function doesn't enter an infinite loop
        let mut iterations = 0;
        let iterations_max = 1e5 as u64;

        while i < self.nonspectral_units.len() {
            iterations += 1;
            assert!(iterations < iterations_max);
            // verify that nsu[i] doesn't overlap with any walls
            let mut nsu_i = self.obj_pool.movable[self.nonspectral_units[i].idx as usize].as_mut().unwrap();
            // [start, end). Note that half-open interval. Use f32s to prevent underflows (bounding boxes may have
            // negative bounds)
            let xstart = f32::clamp(nsu_i.bounding_box.x1, 0.0, self.wall_x_end as f32) as usize;
            let xend = f32::clamp(nsu_i.bounding_box.x2 + 1.0, 0.0, self.wall_x_end as f32) as usize;
            let ystart = f32::clamp(nsu_i.bounding_box.y1, 0.0, self.wall_y_end as f32) as usize;
            let yend = f32::clamp(nsu_i.bounding_box.y2 + 1.0, 0.0, self.wall_y_end as f32) as usize;
            let mut bw_loop_iter = 0;
            let mut end_bw_check_loop = false;
            while !end_bw_check_loop {
                bw_loop_iter += 1;
                assert!(bw_loop_iter < iterations_max);

                end_bw_check_loop = true;
                'outer: for x in xstart..xend {
                    for y in ystart..yend {
                        iterations += 1;
                        if let Some(ref bw) = self.has_wall_at_coordinate[x][y] {
                            let bw = self.obj_pool.basic_walls[bw.idx as usize].as_ref().unwrap();
                            if nsu_i.overlaps_ro_wall(&bw) {
                                collisions.push(RofizCollision::new(nsu_i.room_object_ref, false, bw.room_object_ref, false));
                                // keep moving the unit back while both of the following hold:
                                // 1. the unit is moved back to a different position
                                // 2. the different position overlaps with a wall.
                                // Remember, we assert that the unit's original position must never overlap with a basic wall
                                while Self::move_back(&mut nsu_i) && nsu_i.overlaps_ro_wall(&bw) {
                                    iterations += 1;
                                    assert!(iterations < iterations_max);
                                }
                                
                                // since the unit moved, it needs to be rechecked against all walls.
                                end_bw_check_loop = false;
                                break 'outer;
                            }
                        }
                    }
                }
            }

            let mut i_override = None;
            let mut bbo_idx = 0;
            while bbo_idx < nsu_bb_overlap[i].len() {
                iterations += 1;
                assert!(iterations < iterations_max);
                let j = nsu_bb_overlap[i][bbo_idx];
                if j > i {
                    break;
                }
                bbo_idx += 1;

                let (mut nsu_i, mut nsu_j) = self.obj_pool.get_mo_mo_mut(&self.nonspectral_units[i], &self.nonspectral_units[j]);
                if nsu_i.overlaps_ro_movable(&nsu_j) {
                    collisions.push(RofizCollision::new(nsu_i.room_object_ref, false, nsu_j.room_object_ref, false));

                    // if this collision can be solved by only one of nsu_i and one of nsu_j moving back, do that.
                    // This is to avoid deadlock, e.g. object A is right above object B. Object A's velocity is
                    // (2, 1) and object B's velocity is (2, 2). In this case, object A should ideally move fully, and
                    // object B might be able to move to a fallback location (rather than stay in its original place).

                    // check if this collision can be solved just by moving nsu_i back. If so, only move nsu_i back.
                    // nsu_i will need to be rechecked against all walls.
                    if !nsu_i.initial_overlaps_ro_movable(&nsu_j) {
                        while Self::move_back(&mut nsu_i) && nsu_i.overlaps_ro_movable(&nsu_j) {
                            iterations += 1;
                            assert!(iterations < iterations_max);
                        }
                        i_override = Some(i);
                        break;
                    }

                    // check if this collision can be solved just by moving nsu_j back. If so, only move nsu_j back.
                    if !nsu_j.initial_overlaps_ro_movable(&nsu_i) {
                        while Self::move_back(&mut nsu_j) && nsu_i.overlaps_ro_movable(&nsu_j) {
                            iterations += 1;
                            assert!(iterations < iterations_max);
                        }
                        i_override = Some(j);
                        break;
                    }

                    // Otherwise, move both of them back. Note that we can't just move nsu_i back to its original location,
                    // because even though nsu_i's original location intersects with nsu_j's current temp location, one
                    // of nsu_i's intermediate fallback locations might not intersect with nsu_j.
                    while Self::move_back(&mut nsu_i) && nsu_i.overlaps_ro_movable(&nsu_j) {
                        iterations += 1;
                        assert!(iterations < iterations_max);
                    }

                    if nsu_i.overlaps_ro_movable(&nsu_j) {
                        while Self::move_back(&mut nsu_j) && nsu_i.overlaps_ro_movable(&nsu_j) {
                            iterations += 1;
                            assert!(iterations < iterations_max);
                        }
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
        log::trace!("Rofiz moafc4 iterations={}", iterations);
    }

    #[inline(never)]
    fn moafc5(&mut self) -> Vec<RofizObjPoolRef> {
        let mut spatial_grid_id_to_obj = Vec::new();
        self.spatial_grid.iter_mut().for_each(|column| column.iter_mut().for_each(|cell| cell.clear()));
        for i in 0..self.nonspectral_units.len() {
            let nsu_i = self.obj_pool.get_mo_mut(&self.nonspectral_units[i]);
            // [start, end). Note that half-open interval. Use f32s to prevent underflows (bounding boxes may have
            // negative bounds)
            let xstart = f32::clamp(nsu_i.bounding_box.x1, 0.0, self.wall_x_end as f32) as usize;
            let xend = f32::clamp(nsu_i.bounding_box.x2 + 1.0, 0.0, self.wall_x_end as f32) as usize;
            let ystart = f32::clamp(nsu_i.bounding_box.y1, 0.0, self.wall_y_end as f32) as usize;
            let yend = f32::clamp(nsu_i.bounding_box.y2 + 1.0, 0.0, self.wall_y_end as f32) as usize;

            for x in xstart..xend {
                for y in ystart..yend {
                    self.spatial_grid[x][y].push(spatial_grid_id_to_obj.len());
                }
            }
            spatial_grid_id_to_obj.push(self.nonspectral_units[i].clone());
        }
        spatial_grid_id_to_obj
    }

    #[inline(never)]
    fn moafc6(&mut self, collisions: &mut Vec<RofizCollision>, spatial_grid_id_to_obj: &mut Vec<RofizObjPoolRef>) {
        let mut collision_candidates = Vec::<usize>::new();
        for (i, mo_rc) in self.spectral_units.iter().chain(self.basic_projectiles.iter()).enumerate() {
            collision_candidates.clear();
            let mo = self.obj_pool.movable[mo_rc.idx as usize].as_ref().unwrap();

            // [start, end). Note that half-open interval. Use f32s to prevent underflows (bounding boxes may have
            // negative bounds)
            let xstart = f32::clamp(mo.bounding_box.x1, 0.0, self.wall_x_end as f32) as usize;
            let xend = f32::clamp(mo.bounding_box.x2 + 1.0, 0.0, self.wall_x_end as f32) as usize;
            let ystart = f32::clamp(mo.bounding_box.y1, 0.0, self.wall_y_end as f32) as usize;
            let yend = f32::clamp(mo.bounding_box.y2 + 1.0, 0.0, self.wall_y_end as f32) as usize;
            for x in xstart..xend {
                for y in ystart..yend {
                    if let Some(ref bw) = self.has_wall_at_coordinate[x][y] {
                        let bw = self.obj_pool.basic_walls[bw.idx as usize].as_ref().unwrap();
                        if mo.overlaps_ro_wall(&bw) {
                            collisions.push(RofizCollision::new(mo.room_object_ref, true, bw.room_object_ref, false));
                        }
                    }

                    collision_candidates.extend(self.spatial_grid[x][y].iter());
                }
            }

            collision_candidates.sort_unstable();
            collision_candidates.dedup();

            for sg_idx in collision_candidates.iter() {
                let sgo = self.obj_pool.movable[spatial_grid_id_to_obj[*sg_idx].idx as usize].as_ref().unwrap();
                if mo.overlaps_ro_movable(&sgo) {
                    collisions.push(RofizCollision::new(mo.room_object_ref, true, sgo.room_object_ref, sgo.is_spectral));
                }
            }

            // add all spectral units to the spatial grid
            if i < self.spectral_units.len() {
                for x in xstart..xend {
                    for y in ystart..yend {
                        self.spatial_grid[x][y].push(spatial_grid_id_to_obj.len());
                    }
                }
                spatial_grid_id_to_obj.push(*mo_rc);
            }
        }
    }

    #[inline(never)]
    fn moafc7(&mut self) {
        for mo_rc in         self.basic_projectiles.iter()
        .chain(self.spectral_units.iter())
        .chain(self.nonspectral_units.iter()) {
            self.obj_pool.get_mo_mut(mo_rc).officially_move();
        }
    }

    #[inline(never)]
    fn moafc8(&mut self, collisions: &mut Vec<RofizCollision>) {
        // A single RoomObject may own both spectral and nonspectral RofizObjects. 
        // Therefore, we must use a key to dedupe collisions.
        // Additionally, we need to ensure that if the two collisions (a, b) and (b, a) are detected,
        // only one of them survives deduping.
        collisions.sort_unstable_by_key(|x| Self::order_room_obj_refs(x.room_obj_ref1, x.room_obj_ref2));
        collisions.dedup_by_key(|x| Self::order_room_obj_refs(x.room_obj_ref1, x.room_obj_ref2));
    }

    fn order_room_obj_refs<'a>(a: RoomObjectRef, b: RoomObjectRef) -> (RoomObjectRef, RoomObjectRef) {
        if a < b {
            (a, b)
        } else {
            (b, a)
        }
    }

    // returns true if the object was moved back to a different location
    fn move_back(rom: &mut RofizObjMovable) -> bool {
        match rom.movement {
        RofizObjectMovement::MoveWithFallbacks(ref v) => {
            if rom.fallback_idx < v.len() {
                rom.fallback_idx += 1;
                if rom.fallback_idx < v.len() {
                    let new_xform = rom.current.transformation.add(&v[rom.fallback_idx]); 
                    new_xform.replace_shape_with_transformed(&mut rom.temp_hitbox, &rom.current.shape);
                } else {
                    rom.current.transformation.replace_shape_with_transformed(&mut rom.temp_hitbox, &rom.current.shape);
                }
                true
            } else {
                false
            }
        }
        _ => {
            if rom.move_successful {
                rom.move_successful = false;
                rom.current.transformation.replace_shape_with_transformed(&mut rom.temp_hitbox, &rom.current.shape);
                true
            } else {
                false
            }
        }
        }
    }

    fn _movable_objs_iter(&mut self) -> impl Iterator<Item = &RofizObjPoolRef> {
        self.basic_projectiles.iter()
                .chain(self.spectral_units.iter())
                .chain(self.nonspectral_units.iter())
    }

    fn new_rofiz_obj_basic_wall(&mut self, floor_object_id: RoomObjectRef, x: u32, y: u32) -> RofizObjBasicWall {
        self.obj_creation_counter += 1;
        let shape = Shape::of_square(x as f32, y as f32, 1.0);
        let bounding_box = BoundingBox::of_shape(&shape);
        RofizObjBasicWall { 
            id: self.obj_creation_counter, 
            external_ref_count: Arc::new(()),
            room_object_ref: floor_object_id,
            x, 
            y, 
            shape,
            bounding_box,
        }
    }

    fn new_rofiz_obj_movable(&mut self, hitbox: Hitbox, room_object_ref: RoomObjectRef, is_spectral: bool) -> RofizObjMovable {
        self.obj_creation_counter += 1;
        RofizObjMovable { 
            id: self.obj_creation_counter, 
            external_ref_count: Arc::new(()),
            current: hitbox, 
            movement: RofizObjectMovement::NoMove(), 
            move_with_fallbacks_idx: 0,
            is_spectral,

            cached_mem_hitbox: Hitbox::default(),

            bounding_box: BoundingBox::zero_state(),
            temp_hitbox: Shape::dummy(),
            initial_hitbox: Shape::dummy(),
            move_successful: false, // dummy
            fallback_idx: 0, // dummy
            room_object_ref: room_object_ref,
            shape_scratchpad: Shape::dummy(),
        }
    }

    pub fn get_stats(&self) -> RofizStats {
        RofizStats { 
            num_walls: self.basic_walls.len(), 
            num_projectiles: self.basic_projectiles.len(), 
            num_spectral_units: self.spectral_units.len(), 
            num_nonspectral_units: self.nonspectral_units.len(), 
            max_x: self.wall_x_end, 
            max_y: self.wall_y_end,
        }
    }
}

pub struct RofizStats {
    pub num_walls: usize,
    pub num_projectiles: usize,
    pub num_spectral_units: usize,
    pub num_nonspectral_units: usize,
    pub max_x: usize,
    pub max_y: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RofizCollision {
    pub room_obj_ref1: RoomObjectRef,
    pub is1_spectral: bool,
    pub room_obj_ref2: RoomObjectRef,
    pub is2_spectral: bool,
}

impl RofizCollision {
    fn new(room_obj_ref1: RoomObjectRef, is1_spectral: bool, room_obj_ref2: RoomObjectRef, is2_spectral: bool) -> Self {
        Self {
            room_obj_ref1,
            is1_spectral,
            room_obj_ref2,
            is2_spectral,
        }
    }
}
