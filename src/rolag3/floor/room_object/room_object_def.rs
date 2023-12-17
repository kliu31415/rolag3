use std::{rc::{Rc, Weak}, cell::RefCell, collections::{HashSet, HashMap}, ops::Range};

use rand::{rngs::ThreadRng, Rng};

use crate::rolag3::floor::{draw::DrawContext, run::PlayerInput, rofiz::{rofiz_state::{RofizState, RofizObjectRef}, rofiz_object::Hitbox}, room::{Room, RoomConnectionInfo}};

use super::damage::DamageColor;

pub trait RoomObject {
    fn is_player(&self) -> bool {
        false
    }
    fn get_metadata(&self) -> &RoomObjectMetadata;
    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response;
    fn draw(&mut self, ctx: &mut DrawContext);
    fn handle_room_just_cleared(&self, _ctx: &mut HandleRoomJustClearedContext) {
        // I don't think any subclass uses this function right now
        // nop
    }

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse;
    fn handle_collision_projectile(&mut self, _: &HcProjectileContext) -> HcProjectileResponse {
        HcProjectileResponse {
            projectile_consumed: false,
            damage_dealt: 0.0,
            room_objects_to_delete: Vec::new(),
        }
    }
    fn handle_collision_black_hole(&mut self, _: &HcBlackHoleContext) -> HcBlackHoleResponse {
        HcBlackHoleResponse {  room_objects_to_delete: Vec::new() }
    } 

    fn handle_collision_tile(&mut self, _ctx: &HcTileContext) -> HcTileResponse {
        HcTileResponse { unit_affected: false }
    }

    fn handle_room_connection_collision(&mut self, _rci: &RoomConnectionInfo) {
        // nop by default
    }

    fn apply_operation(&mut self, _ctx: &RoomObjApplyOperationContext) {
        // nop by default
    }

    fn get_room_object_type(&self) -> RoomObjectType;

    fn is_spectral(&self) -> bool;
    fn blocks_room_clear(&self) -> bool {
        false
    }
    fn is_wall_at(&self, _x: u32, _y: u32) -> bool {
        false
    }

    fn handle_query_unit_info(&self, _ctx: &RoQueryUnitInfoContext) -> RoQueryUnitInfoResponse {
        unimplemented!("handle_query_unit_info() can only be called for units. Called for {:?}", self.get_room_object_type());
    }
}

pub struct RoomObjApplyOperationContext<'a> {
    operation: &'a RoomObjOperation,
    rofiz: &'a RofizState,
    tick_length: f64,
}

impl<'a> RoomObjApplyOperationContext<'a> {
    pub fn get_operation(&self) -> &RoomObjOperation {
        self.operation
    }

    pub fn get_rofiz(&self) -> &RofizState {
        self.rofiz
    }

    pub fn get_tick_length(&self) -> f64 {
        self.tick_length
    }
}

pub enum RoomObjOperation {
    BlackHoleForce { x: f64, y: f64, colors: Vec<DamageColor>, accel_fn: fn(f64) -> f64 /* dist -> accel */},
}

pub struct RoQueryUnitInfoContext<'a> {
    self_as_weak: Weak<RefCell<dyn RoomObject>>,
    rofiz: &'a RofizState,
}

impl<'a> RoQueryUnitInfoContext<'a> {
    pub fn get_rofiz(&self) -> &RofizState {
        self.rofiz
    }

    pub fn get_self_as_weak(&self) -> Weak<RefCell<dyn RoomObject>> {
        self.self_as_weak.clone()
    }
}

#[derive(Debug)]
pub struct RoQueryUnitInfoResponse {
    pub unit: Weak<RefCell<dyn RoomObject>>,
    pub team: Team,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoomObjectType {
    Wall,
    Projectile,
    Unit,
    Other,
}

pub struct RoomObjectMetadata {
    id: RoomObjectId,
}

pub type RoomObjectId = usize;

impl RoomObjectMetadata {
    pub fn new(ctx: &mut NewRoomObjectContext) -> Self {
        Self { 
            id: ctx.get_next_floor_object_id(),
        }
    }

    pub fn new_for_player() ->Self {
        Self {
            id: Room::PLAYER_ROOM_OBJECT_ID,
        }
    }

    pub fn get_id(&self) -> RoomObjectId {
        self.id
    }
}

pub struct RoomObjectCollection {
    player: Vec<Rc<RefCell<dyn RoomObject>>>, // size 0 (player not present in room) or 1 (player present)
    ro_wall: Vec<Rc<RefCell<dyn RoomObject>>>,
    ro_projectile: Vec<Rc<RefCell<dyn RoomObject>>>,
    ro_unit: Vec<Rc<RefCell<dyn RoomObject>>>,
    ro_other: Vec<Rc<RefCell<dyn RoomObject>>>,
    room_already_cleared: bool,
}

impl RoomObjectCollection {
    pub fn new() -> Self {
        Self {
            player: Vec::new(),
            ro_wall: Vec::new(),
            ro_projectile: Vec::new(),
            ro_unit: Vec::new(),
            ro_other: Vec::new(),
            room_already_cleared: false,
        }
    }

    fn vec_mut_all_objects(&mut self) -> Vec<&mut Vec<Rc<RefCell<dyn RoomObject>>>> {
        vec![&mut self.player, &mut self.ro_wall, &mut self.ro_projectile, &mut self.ro_unit, &mut self.ro_other]
    }

    fn vec_all_objects(&self) -> Vec<&Vec<Rc<RefCell<dyn RoomObject>>>> {
        vec![&self.player, &self.ro_wall, &self.ro_projectile, &self.ro_unit, &self.ro_other]
    }

    fn iter(&self) -> impl Iterator<Item = &Rc<RefCell<dyn RoomObject>>> {
        self.vec_all_objects().into_iter().flatten()
    }

    fn object_count(&self) -> usize {
        self.vec_all_objects().into_iter().map(|v| v.len()).sum()
    }

    pub fn remove_player(&mut self) {
        assert!(self.player.len() == 1, "remove_player() called when player is not present");
        self.player.clear();
    }

    pub fn add(&mut self, obj: Rc<RefCell<dyn RoomObject>>) {
        if obj.borrow().is_player() {
            assert!(self.player.is_empty(), "cannot add player to room when player is already in room");
            self.player.push(obj);
            return;
        }
        let ro_type = obj.borrow().get_room_object_type();
        match ro_type {
            RoomObjectType::Wall => self.ro_wall.push(obj),
            RoomObjectType::Projectile => self.ro_projectile.push(obj),
            RoomObjectType::Unit => self.ro_unit.push(obj),
            RoomObjectType::Other => self.ro_other.push(obj),
        }
    }

    pub fn remove_wall_at(&mut self, x: u32, y: u32) {
        let old_len = self.ro_wall.len();
        self.ro_wall.retain(|ro| !ro.borrow().is_wall_at(x, y));
        let new_len = self.ro_wall.len();
        assert!(old_len == new_len + 1, "Tried to remove wall at (x, y) = ({}, {}) from RoomObjectCollection. \
            Expected to remove one object. old_len={}, new_len={}", x, y, old_len, new_len);
    }

    pub fn act1(&mut self, ctx: &mut Act1Context) {
        let mut responses: Vec<Act1Response> = Vec::new();
        self.iter().for_each(|x| {
            ctx.self_as_rc = Some(x.clone());
            responses.push(x.borrow_mut().act1(ctx))}
        );

        let should_remove: Vec<bool> = responses.iter().map(|x| x.get_remove_me()).collect();
        self.remove_all_using_bool_array(should_remove.as_slice());

        responses.iter_mut().flat_map(|x| x.steal_room_objs_to_add()).for_each(|x| self.add(x));

        let operations = responses.iter_mut().flat_map(|x| x.steal_operations());
        for op in operations {
            let op_ctx = RoomObjApplyOperationContext {
                operation: &op,
                rofiz: ctx.rofiz,
                tick_length: ctx.tick_length,
            };
            match op {
                RoomObjOperation::BlackHoleForce { .. } => {
                    self.ro_projectile.iter().for_each(|x| x.borrow_mut().apply_operation(&op_ctx))
                },
            }
        }

        let queries = responses.iter_mut().flat_map(|x| x.steal_queries());
        for (qargs, qresult) in queries {
            match qargs {
                Act1QueryArgs::ClosestUnit { x, y, team_filter } => {
                    let closest = self.ro_unit.iter().chain(self.player.iter())
                        .map(|x| {
                            let rqui_ctx = RoQueryUnitInfoContext {
                                self_as_weak: Rc::downgrade(x),
                                rofiz: ctx.rofiz,
                            };
                            x.borrow().handle_query_unit_info(&rqui_ctx)
                        })
                        .filter(|x| team_filter.is_none() || team_filter.unwrap() == x.team)
                        .min_by(|a, b| {
                            let dist_a = f64::powi(a.x - x, 2) + f64::powi(a.y - y, 2);
                            let dist_b = f64::powi(b.x - x, 2) + f64::powi(b.y - y, 2);
                            dist_a.partial_cmp(&dist_b).unwrap()
                        });
                    match closest {
                        Some(c) => qresult.replace(
                            Act1QueryResult::ClosestUnit(Some(UnitInfo { 
                                unit: c.unit,
                                distance: f64::hypot(c.x - x, c.y - y),
                                team: c.team,
                                x: c.x, 
                                y: c.y,
                            }))
                        ),
                        None => qresult.replace(Act1QueryResult::ClosestUnit(None)),
                    };
                }
            }
        }
    }

    pub fn remove_by_id(&mut self, to_remove: HashSet<RoomObjectId>) {
        if to_remove.is_empty() {
            return;
        }
        self.vec_mut_all_objects().iter_mut().for_each(
            |ro_v| ro_v.retain(
                |x| !to_remove.contains(&x.borrow().get_metadata().get_id())));
    }

    pub fn draw(&mut self, ctx: &mut DrawContext) {
        for fo in self.iter() {
            fo.borrow_mut().draw(ctx);
        }
    }

    pub fn handle_if_room_just_cleared(&mut self, rofiz: &mut RofizState) {
        if self.room_already_cleared {
            return;
        }

        for fo in self.iter() {
            if fo.borrow().blocks_room_clear() {
                return;
            }
        }

        self.room_already_cleared = true;

        let mut ctx = HandleRoomJustClearedContext {
            _rofiz: rofiz,
        };
        for fo in self.iter() {
            fo.borrow_mut().handle_room_just_cleared(&mut ctx);
        }
    }

    pub fn is_room_cleared(&self) -> bool {
        self.room_already_cleared
    }

    pub fn remove_all_using_bool_array(&mut self, should_remove: &[bool]) {
        let obj_count = self.object_count();
        if should_remove.len() != obj_count {
            panic!("should_remove.len() != obj_count. Values: {} != {}", should_remove.len(), obj_count)
        }
        let mut idx = 0;
        self.vec_mut_all_objects().iter_mut().for_each(|v| v.retain(|_| {idx += 1; !should_remove[idx - 1]}));
    }

    pub fn get_multi(&self, ids: HashSet<RoomObjectId>) -> HashMap<RoomObjectId, Rc<RefCell<dyn RoomObject>>> {
        let mut res = HashMap::new();
        for fo in self.iter() {
            let id = fo.borrow().get_metadata().get_id();
            if ids.contains(&id) {
                res.insert(id, fo.clone());
            }
        }
        res
    }

    pub fn validate(&self) {
        for ro in self.iter() {
            let ref_count = Rc::strong_count(ro);
            if ro.borrow().is_player() {
                if ref_count != 3 {
                    panic!("RoomObject Player Rc::strong_count()={}. Expected 3. Id={:?}", ref_count, ro.borrow().get_metadata().get_id());
                }
            } else if ref_count != 1 {
                panic!("RoomObject Rc::strong_count()={}. Expected 1. Id={:?}", ref_count, ro.borrow().get_metadata().get_id());
            }
        }
    }
}

pub struct NewRoomObjectContext<'a> {
    rofiz: &'a mut RofizState,
    room_object_id_counter: &'a mut RoomObjectId,
    room_time: f64,
    rng: &'a mut ThreadRng,
}

impl<'a> NewRoomObjectContext<'a> {
    pub fn new(rofiz: &'a mut RofizState, room_object_id_counter: &'a mut RoomObjectId, room_time: f64, rng: &'a mut ThreadRng,) -> Self {
        Self {
            rofiz,
            room_object_id_counter,
            room_time,
            rng,
        }
    }

    pub fn from_act1_ctx(act1_ctx: &'a mut Act1Context) -> Self {
        Self {
            rofiz: act1_ctx.rofiz,
            room_object_id_counter: act1_ctx.room_object_id_counter,
            room_time: act1_ctx.room_time,
            rng: act1_ctx.rng,
        }
    }

    pub fn get_next_floor_object_id(&mut self) -> RoomObjectId {
        *self.room_object_id_counter += 1;
        *self.room_object_id_counter
    }

    pub fn get_room_time(&mut self) -> f64 {
        self.room_time
    }
    
    // in the range [0, 1)
    pub fn get_randf64(&mut self) -> f64 {
        self.rng.gen::<f64>()
    }

    pub fn _get_randu64(&mut self, r: Range<u64>) -> u64 {
        self.rng.gen_range(r)
    }

    pub fn get_randi64(&mut self, r: Range<i64>) -> i64 {
        self.rng.gen_range(r)
    }

    pub fn add_basic_wall(&mut self, floor_object_id: RoomObjectId, x: u32, y: u32) -> RofizObjectRef {
        self.rofiz.add_basic_wall(floor_object_id, x, y)
    }

    pub fn add_nonspectral_unit(&mut self, floor_object_id: RoomObjectId, hitbox: Hitbox) -> RofizObjectRef {
        self.rofiz.add_nonspectral_unit(floor_object_id, hitbox)
    }

    pub fn add_spectral_unit(&mut self, floor_object_id: RoomObjectId, hitbox: Hitbox) -> RofizObjectRef {
        self.rofiz.add_spectral_unit(floor_object_id, hitbox)
    }

    pub fn add_basic_projectile(&mut self, floor_object_id: RoomObjectId, hitbox: Hitbox) -> RofizObjectRef {
        self.rofiz.add_basic_projectile(floor_object_id, hitbox)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FloorCoordinate {
    pub x: f64,
    pub y: f64,
}

impl FloorCoordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self {x, y}
    }
}

pub struct Act1Context<'a> {
    player_input: &'a PlayerInput,
    rofiz: &'a mut RofizState,
    room_object_id_counter: &'a mut RoomObjectId,
    self_as_rc: Option<Rc<RefCell<dyn RoomObject>>>,
    tick_length: f64,
    room_time: f64,
    rng: &'a mut ThreadRng,
    room_cleared_at_time: Option<f64>,
}

impl<'a> Act1Context<'a> {
    pub fn new(
        player_input: &'a PlayerInput, 
        rofiz: &'a mut RofizState, 
        room_object_id_counter: &'a mut RoomObjectId, 
        tick_length: f64, 
        room_time: f64,
        rng: &'a mut ThreadRng,
        room_cleared_at_time: Option<f64>,
    ) -> Self {
        Self {
            player_input,
            rofiz,
            room_object_id_counter,
            self_as_rc: Option::None,
            tick_length,
            room_time,
            rng,
            room_cleared_at_time,
        }
    }

    pub fn get_player_input(&self) -> &PlayerInput {
        self.player_input
    }

    pub fn get_rofiz(&mut self) -> &mut RofizState {
        self.rofiz
    }

    pub fn get_tick_length(&self) -> f64 {
        self.tick_length
    }

    pub fn get_room_time(&self) -> f64 {
        self.room_time
    }

    pub fn get_self_as_weak(&self) -> Weak<RefCell<dyn RoomObject>> {
        Rc::downgrade(&self.self_as_rc.clone().unwrap())
    }

    // in the range [0, 1)
    pub fn get_randf64(&mut self) -> f64 {
        self.rng.gen::<f64>()
    }

    pub fn get_room_cleared_at_time(&self) -> Option<f64> {
        self.room_cleared_at_time
    }
}

pub enum Act1QueryArgs {
    ClosestUnit{x: f64, y: f64, team_filter: Option<Team>}
}

#[derive(Debug)]
pub enum Act1QueryResult {
    NotSet,
    ClosestUnit(Option<UnitInfo>),
}

#[derive(Debug)]
pub struct UnitInfo {
    pub unit: Weak<RefCell<dyn RoomObject>>,
    pub distance: f64,
    pub team: Team,
    pub x: f64,
    pub y: f64,
}

pub struct Act1Response {
    should_remove_me: bool,
    objects_to_add: Vec<Rc<RefCell<dyn RoomObject>>>,
    act1_queries: Vec<(Act1QueryArgs, Rc<RefCell<Act1QueryResult>>)>,
    operations: Vec<RoomObjOperation>,
}

impl Act1Response {
    pub fn new() -> Self {
        Self {
            should_remove_me: false,
            objects_to_add: Vec::new(),
            act1_queries: Vec::new(),
            operations: Vec::new(),
        }
    }

    pub fn remove_me(mut self) -> Self {
        self.should_remove_me = true;
        self
    }

    pub fn get_remove_me(&self) -> bool {
        self.should_remove_me
    }

    pub fn add_room_obj(&mut self, obj: Rc<RefCell<dyn RoomObject>>) {
        self.objects_to_add.push(obj);
    }

    pub fn steal_room_objs_to_add(&mut self) -> Vec<Rc<RefCell<dyn RoomObject>>> {
        let mut empty_vec = Vec::new();
        std::mem::swap(&mut empty_vec, &mut self.objects_to_add);
        empty_vec
    }

    pub fn add_query(&mut self, args: Act1QueryArgs) -> Rc<RefCell<Act1QueryResult>> {
        let rc_result = Rc::new(RefCell::new(Act1QueryResult::NotSet));
        self.act1_queries.push((args, rc_result.clone()));
        rc_result
    }

    pub fn steal_queries(&mut self) -> Vec<(Act1QueryArgs, Rc<RefCell<Act1QueryResult>>)> {
        let mut q = Vec::new();
        std::mem::swap(&mut q, &mut self.act1_queries);
        q
    }

    pub fn apply_operation(&mut self, op: RoomObjOperation) {
        self.operations.push(op);
    }

    pub fn steal_operations(&mut self) -> Vec<RoomObjOperation> {
        let mut q = Vec::new();
        std::mem::swap(&mut q, &mut self.operations);
        q
    }
}

pub struct HandleRoomJustClearedContext<'a> {
    _rofiz: &'a mut RofizState,
}

impl<'a> HandleRoomJustClearedContext<'a> {
    pub fn _get_rofiz(&mut self) -> &mut RofizState {
        self._rofiz
    }
}

pub struct HandleCollisionContext<'a> {
    other: Rc<RefCell<dyn RoomObject>>,
    rng: &'a mut ThreadRng,
    room_time: f64,
    _tick_length: f64,
    _rofiz: &'a RofizState,
}

impl<'a> HandleCollisionContext<'a> {
    pub fn new(other: Rc<RefCell<dyn RoomObject>>, rng: &'a mut ThreadRng, room_time: f64, tick_length: f64, rofiz: &'a RofizState) -> Self {
        Self { 
            other,
            rng,
            room_time,
            _tick_length: tick_length,
            _rofiz: rofiz,
        }
    }
    
    pub fn get_other(&mut self) -> Rc<RefCell<dyn RoomObject>> {
        self.other.clone()
    }

    // in the range [0, 1)
    pub fn _get_randf64(&mut self) -> f64 {
        self.rng.gen::<f64>()
    }

    pub fn _get_randu64(&mut self, r: Range<u64>) -> u64 {
        self.rng.gen_range(r)
    }

    pub fn get_randi64(&mut self, r: Range<i64>) -> i64 {
        self.rng.gen_range(r)
    }

    pub fn get_room_time(&self) -> f64 {
        self.room_time
    }

    pub fn _get_tick_length(&self) -> f64 {
        self._tick_length
    }

    pub fn _get_rofiz(&self) -> &RofizState {
        self._rofiz
    }
}

pub struct HandleCollisionResponse {
    room_objects_to_remove: Vec<RoomObjectId>,
}

impl HandleCollisionResponse {
    pub fn new() -> Self {
        Self { 
            room_objects_to_remove: Vec::new(),
        }
    }

    pub fn remove_room_obj(mut self, id: RoomObjectId) -> Self {
        self.room_objects_to_remove.push(id);
        self
    }

    pub fn remove_room_objs(mut self, ids: &[RoomObjectId]) -> Self {
        self.room_objects_to_remove.extend(ids);
        self
    }

    pub fn get_room_objects_to_remove(&self) -> &[RoomObjectId] {
        self.room_objects_to_remove.as_slice()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Team {
    Player,
    Enemy,
}

pub struct HcProjectileContext {
    pub team: Team,
    pub damage_color: DamageColor,
    pub damage: f64,
    pub room_time: f64,
}

pub struct HcProjectileResponse {
    pub projectile_consumed: bool,
    pub damage_dealt: f64,
    pub room_objects_to_delete: Vec<RoomObjectId>,
}

impl HcProjectileResponse {
    pub fn nop() -> Self {
        Self {
            projectile_consumed: false,
            damage_dealt: 0.0,
            room_objects_to_delete: Vec::new(),
        }
    }
}

pub struct HcBlackHoleContext {
    pub affects_projectiles_color_filter: Option<DamageColor>,
}

pub struct HcBlackHoleResponse {
    pub room_objects_to_delete: Vec<RoomObjectId>,
}

pub struct HcTileContext {
    pub tile_effect: HcTileEffect,
}

#[derive(Debug, Clone, Copy)]
pub enum HcTileEffect {
    Accelerate {force: f64, theta: f64}
}

pub struct HcTileResponse {
    pub unit_affected: bool,
}