use std::{rc::{Rc, Weak}, cell::RefCell, collections::{HashSet, HashMap, BTreeMap}, ops::Range};

use crate::{rolag3::{floor::{draw::DrawContext, run::PlayerInput, rofiz::{rofiz_state::{RofizState, RofizObjectRef}, rofiz_object::Hitbox}, room::{RoomConnectionInfo, RoomTile}, floor_def::Floor, floorgen::run::GenFloorRoomContext}, sound_db::SoundDb}, util::rng::Prng, sfx::sound_system::{SoundSystem, PlaySoundArgs, SoundDataRef}};

use super::{damage::DamageColor, unit::standard_unit_common::{Budeb, StandardUnitCommon}, sound::{RoomObjSound, RoomObjPlaySoundArgs, RoomObjSoundRef, RoomObjSoundIdT, RoomObjPlaySoundArgsBuilder, RoomObjPlaySoundArgsBuilderReq}};

/* Rules:
   -act1() must be called at least once before any draw() calls. This allows initialization steps to be performed in
    act1(). 
*/

pub trait RoomObject {
    fn is_player(&self) -> bool {
        false
    }
    fn get_metadata(&self) -> &RoomObjectMetadata;
    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response;
    fn draw(&mut self, ctx: &mut DrawContext);

    fn handle_room_just_cleared(&mut self, _ctx: &mut HandleRoomJustClearedContext) {
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
    fn handle_collision_standard_unit<'a>(&mut self, _: &mut HcStandardUnitContext<'a>) -> HcStandardUnitResponse {
        HcStandardUnitResponse {
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

    fn blocks_projectiles(&self) -> bool {
        false
    }
    fn blocks_room_clear(&self) -> bool {
        false
    }
    fn get_as_wall_location(&self) -> Option<(u32, u32)> {
        None
    }
    fn add_as_wall_location_to(&self, _locs: &mut Vec<(u32, u32)>) {
        // nop by default
    }
    fn add_as_ground_location_to(&self, _locs: &mut Vec<(u32, u32)>) {
        // nop by default
    }

    fn handle_query_unit_info(&self, _ctx: &RoQueryUnitInfoContext) -> Option<RoQueryUnitInfoResponse> {
        unimplemented!("handle_query_unit_info() can only be called for units. Called for {:?}", self.get_metadata().get_ref());
    }

    fn get_as_boss_hp(&mut self) -> BossHp {
        unimplemented!("get_as_boss_hp() not implemented for {:?}", self.get_metadata().get_ref())
    }
}

pub enum BossHp {
    Basic{cur_hp: f64, max_hp: f64}
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
    ClearProjectiles {exclude_teams_filter: Vec<Team> },
    UnitBudeb {exclude_teams_filter: Vec<Team>, budeb: Budeb},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RoomObjectType {
    Wall,
    Projectile,
    Unit,
    Other,
}

pub struct RoomObjectMetadata {
    ref_: RoomObjectRef,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct RoomObjectRef {
    pub typ: RoomObjectType, // used solely as an optimization
    pub id: RoomObjectId, // guaranteed to be unique across all RoomObjects of all types
}

pub type RoomObjectId = usize;

impl RoomObjectMetadata {
    pub fn new(ctx: &mut NewRoomObjectContext, typ: RoomObjectType) -> Self {
        Self { 
            ref_: RoomObjectRef {id: ctx.get_and_inc_next_room_object_id(), typ },
        }
    }

    pub fn new_for_player() ->Self {
        Self {
            ref_: Floor::PLAYER_ROOM_OBJECT_REF,
        }
    }

    pub fn get_ref(&self) -> RoomObjectRef {
        self.ref_
    }
}

pub struct RoomObjectCollection {
    room_already_cleared: bool,
    room_objects_by_type: RoomObjectsByType,

    cached_mem: RoomObjectCollectionCachedMem,
}

struct RoomObjectsByType {
    room_objects: BTreeMap<RoomObjectRef, RoomObjWithEmd>,
}

// Emd = "External" metadata. This is different from RoomObjectMetadata, which is "Internal" metadata
struct RoomObjWithEmd {
    room_obj: Rc<RefCell<dyn RoomObject>>,
    playing_sounds: Vec<RoomObjSound>,
}

impl RoomObjWithEmd {
    fn new(room_obj: Rc<RefCell<dyn RoomObject>>) -> Self {
        Self {
            room_obj,
            playing_sounds: Vec::new(),
        }
    }
}

fn range_all_of_type(t: RoomObjectType) -> Range<RoomObjectRef> {
    let a = RoomObjectRef {
        id: RoomObjectId::MIN,
        typ: t,
    };
    let b = RoomObjectRef {
        id: RoomObjectId::MAX,
        typ: t,
    };
    a..b
}

impl RoomObjectsByType {
    fn _object_count(&self) -> usize {
        self.room_objects.len()
    }

    fn add(&mut self, obj: Rc<RefCell<dyn RoomObject>>) {
        let ref_ = obj.as_ref().borrow().get_metadata().get_ref();
        let old = self.room_objects.insert(ref_, RoomObjWithEmd::new(obj));
        assert!(old.is_none(), "tried to add RoomObject (id={:?}) to RoomObjectsByType when a RoomObject with the same id already exists", ref_);
    }

    fn remove(&mut self, r: &RoomObjectRef) {
        let removed = self.room_objects.remove(r);
        if let Some(removed) = removed {
            if !removed.room_obj.borrow().is_player() {
                let v = Rc::strong_count(&removed.room_obj);
                assert_eq!(1, v,"after removing room object with id={:?}, Rc strong count is {} (expected 1)", r, v);
            }
        } else {
            panic!("unable to remove room object with id={:?}", r);
        }
    }

    fn remove_wall_at(&mut self, x: u32, y: u32, expected: Range<usize>) {
        let to_remove = self.room_objects.range_mut(range_all_of_type(RoomObjectType::Wall)).filter_map(|(k, v)| {
            let wall_loc = v.room_obj.as_ref().borrow().get_as_wall_location();
            match wall_loc {
                Some((wx, wy)) => {
                    if x==wx && y==wy {
                        return Some(*k);
                    }
                    return None;
                },
                None => None,
            }
        }).collect::<Vec<_>>();
        assert!(expected.contains(&to_remove.len()), 
            "Tried to remove wall at (x, y) = ({}, {}) from RoomObjectCollection. 
            Expected to remove {:?} objects. Got {} objects", x, y, expected, to_remove.len());
        for id in to_remove {
            self.room_objects.remove(&id).expect("unable to remove RoomObject");
        }
    }

    fn get_wall_locations(&self) -> Vec<(u32, u32)> {
        let mut locs = Vec::new();
        self.room_objects.values().for_each(|x| x.room_obj.borrow().add_as_wall_location_to(&mut locs));
        locs
    }

    fn get_ground_locations(&self) -> Vec<(u32, u32)> {
        let mut locs = Vec::new();
        self.room_objects.values().for_each(|x| x.room_obj.borrow().add_as_ground_location_to(&mut locs));
        locs
    }
}

struct RoomObjectCollectionCachedMem {
    act1_responses: Vec<Act1Response>,
    room_objs_to_add: Vec<Rc<RefCell<dyn RoomObject>>>,
    room_objs_to_remove: Vec<RoomObjectRef>,
    operations: Vec<RoomObjOperation>,
    queries: Vec<(Act1QueryArgs, Rc<RefCell<Act1QueryResult>>)>,
}

impl RoomObjectCollectionCachedMem {
    fn new() -> Self {
        Self {
            act1_responses: Vec::new(),
            room_objs_to_add: Vec::new(),
            room_objs_to_remove: Vec::new(),
            operations: Vec::new(),
            queries: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.act1_responses.clear();
        self.room_objs_to_add.clear();
        self.room_objs_to_remove.clear();
        self.operations.clear();
        self.queries.clear();
    }
}

pub struct RocAct1Response {
    pub floor_finished: bool,
}

impl RoomObjectCollection {
    pub fn new() -> Self {
        Self {
            room_objects_by_type: RoomObjectsByType {
                room_objects: BTreeMap::new(),
            },
            room_already_cleared: false,
            cached_mem: RoomObjectCollectionCachedMem::new(),
        }
    }

    pub fn remove_player(&mut self) {
        self.room_objects_by_type.remove(&Floor::PLAYER_ROOM_OBJECT_REF);
    }

    pub fn add(&mut self, obj: Rc<RefCell<dyn RoomObject>>) {
        self.room_objects_by_type.add(obj);
    }

    pub fn remove_wall_at(&mut self, x: u32, y: u32, expected: Range<usize>) {
        self.room_objects_by_type.remove_wall_at(x, y, expected);
    }

    pub fn get_wall_locations(&self) -> Vec<(u32, u32)> {
        self.room_objects_by_type.get_wall_locations()
    }

    pub fn get_ground_locations(&self) -> Vec<(u32, u32)> {
        self.room_objects_by_type.get_ground_locations()
    }

    pub fn validate_start_room(&self) {
        let mut unique_locations = HashSet::new();
        self.room_objects_by_type.room_objects.range(range_all_of_type(RoomObjectType::Wall)).filter_map(|(_, w)| w.room_obj.as_ref().borrow().get_as_wall_location()).for_each( |loc| {
            assert!(!unique_locations.contains(&loc), "multiple walls detected at location ({}, {})", loc.0, loc.1);
            unique_locations.insert(loc);
        });
    }

    #[inline(never)]
    pub fn act1(&mut self, mut ctx: Act1Context, sound_system: &mut dyn SoundSystem) -> RocAct1Response {
        self.cached_mem.reset();

        for remd in self.room_objects_by_type.room_objects.values_mut() {
            ctx.self_as_rc = Some(remd.room_obj.clone());
            let mut response = remd.room_obj.borrow_mut().act1(&mut ctx);
            let newly_played_sounds = response.take_newly_played_sounds();
            let owned_sound_playback_speed_override = response.take_owned_sound_playback_speed_override();
            self.cached_mem.act1_responses.push(response);

            for nps in newly_played_sounds {
                assert!(nps.volume >= 0.0, "sound volume ({}) is less than 0", nps.volume);
                if nps.volume == 0.0 {
                    continue;
                }
                let r = sound_system.play_sound(PlaySoundArgs {
                    sdr: nps.sound_data,
                    volume: nps.volume,
                    playback_speed: nps.playback_speed,
                    panning: 0.5, // TODO: actually use panning rather than just centering at 0.5
                });
                
                let spr = match r {
                    Ok(v) => v,
                    Err(e) => panic!("unable to play sound. Error = {}", e),
                };

                if nps.owned {
                    remd.playing_sounds.push(RoomObjSound {
                        id: nps.id,
                        location: nps.location,
                        sound_ref: spr,
                    });
                } else {
                    todo!("non-owned RoomObject sounds are not supported yet");
                }
            }

            if let Some(_v) = owned_sound_playback_speed_override {
                todo!("owned_sound_playback_speed_override not supported yet")
            }

            remd.playing_sounds.retain(|x| !sound_system.is_done_playing(&x.sound_ref));

            // TODO: dynamically pan sound every frame based on the sound location and player location
        }
        // Set to None to be extra-safe about enforcing the invariant that the Rc strong count to a RoomObject is 
        // (almost) always 1. This will help prevent bugs.
        ctx.self_as_rc = None;

        let mut floor_finished = false;
        for r in self.cached_mem.act1_responses.iter_mut() {
            r.steal_room_objs_to_add(&mut self.cached_mem.room_objs_to_add);
            r.steal_room_objs_to_remove(&mut self.cached_mem.room_objs_to_remove);
            r.steal_operations(&mut self.cached_mem.operations);
            r.steal_queries(&mut self.cached_mem.queries);
            floor_finished |= r.get_is_floor_finished();
        }
        self.cached_mem.room_objs_to_remove.drain(..).for_each(|x| self.room_objects_by_type.remove(&x));
        self.cached_mem.room_objs_to_add.drain(..).for_each(|x| self.room_objects_by_type.add(x));

        for op in self.cached_mem.operations.drain(..) {
            let op_ctx = RoomObjApplyOperationContext {
                operation: &op,
                rofiz: ctx.rofiz,
                tick_length: ctx.tick_length,
            };
            match op {
                RoomObjOperation::BlackHoleForce { .. } => {
                    self.room_objects_by_type.room_objects.range(range_all_of_type(RoomObjectType::Projectile)).for_each(|(_, v)| v.room_obj.borrow_mut().apply_operation(&op_ctx))
                },
                RoomObjOperation::ClearProjectiles { .. } => {
                    self.room_objects_by_type.room_objects.range(range_all_of_type(RoomObjectType::Projectile)).for_each(|(_, v)| v.room_obj.borrow_mut().apply_operation(&op_ctx))
                },
                RoomObjOperation::UnitBudeb { .. } => {
                    self.room_objects_by_type.room_objects.range(range_all_of_type(RoomObjectType::Unit)).for_each(|(_, v)| v.room_obj.borrow_mut().apply_operation(&op_ctx));
                }
            }
        }

        for (qargs, qresult) in self.cached_mem.queries.drain(..) {
            match qargs {
                Act1QueryArgs::ClosestUnit { x, y, team_filter } => {
                    let closest = self.room_objects_by_type.room_objects.range(range_all_of_type(RoomObjectType::Unit))
                        .map(|(_, v)| {
                            let rqui_ctx = RoQueryUnitInfoContext {
                                self_as_weak: Rc::downgrade(&v.room_obj),
                                rofiz: ctx.rofiz,
                            };
                            v.room_obj.as_ref().borrow().handle_query_unit_info(&rqui_ctx)
                        })
                        .filter_map(|x| x)
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
        RocAct1Response {
            floor_finished,
        }
    }

    pub fn remove_by_id(&mut self, to_remove: HashSet<RoomObjectRef>) {
        if to_remove.is_empty() {
            return;
        }
        for r in to_remove {
            // TODO: should we handle removing the player here?
            self.room_objects_by_type.room_objects.remove(&r).expect(&format!("unable to remove room object with ref={:?}", r));
        }
    }

    pub fn draw(&mut self, ctx: &mut DrawContext) {
        for fo in self.room_objects_by_type.room_objects.values() {
            fo.room_obj.borrow_mut().draw(ctx);
        }
    }

    #[inline(never)]
    pub fn handle_if_room_just_cleared(
        &mut self, 
        rofiz: &mut RofizState, 
        room_time: f64,
        starcash_reward: f64,
    ) {
        if self.room_already_cleared {
            return;
        }

        for fo in self.room_objects_by_type.room_objects.values() {
            if fo.room_obj.as_ref().borrow().blocks_room_clear() {
                return;
            }
        }

        self.room_already_cleared = true;

        let mut ctx = HandleRoomJustClearedContext {
            _rofiz: rofiz,
            room_time,
            starcash_reward,
        };
        for fo in self.room_objects_by_type.room_objects.values() {
            fo.room_obj.borrow_mut().handle_room_just_cleared(&mut ctx);
        }
    }

    pub fn is_room_cleared(&self) -> bool {
        self.room_already_cleared
    }

    pub fn _remove_all_using_bool_array(&mut self, should_remove: &[bool]) {
        let obj_count = self.room_objects_by_type._object_count();
        if should_remove.len() != obj_count {
            panic!("should_remove.len() != obj_count. Values: {} != {}", should_remove.len(), obj_count)
        }
        let mut idx = 0;
        let to_remove = self.room_objects_by_type.room_objects.keys().filter_map(|k| {
            idx += 1;
            if should_remove[idx - 1] {
                return Some(*k);
            }
            return None;
        }).collect::<Vec<_>>();
        for r in to_remove {
            let v = self.room_objects_by_type.room_objects.remove(&r).expect("unable to remove room object");
            assert_eq!(Rc::strong_count(&v.room_obj), 1, "Rc strong count for removed object with id {} isn't 1", v.room_obj.borrow().get_metadata().get_ref().id);
        }
    }

    pub fn _get_multi(&self, ids: HashSet<RoomObjectRef>) -> HashMap<RoomObjectRef, Rc<RefCell<dyn RoomObject>>> {
        let mut res = HashMap::new();
        for id in ids {
            // TODO: if _get_multi is used in the future, the below line needs to be more efficient.
            // Right now, a new formatted string is allocated every loop, which the profiler shows is very slow.
            res.insert(id, self.room_objects_by_type.room_objects.get(&id).expect(&format!("unable to get room object with id={:?}", id)).room_obj.clone());
        }
        res
    }

    pub fn get(&self, id: &RoomObjectRef) -> Rc<RefCell<dyn RoomObject>> {
        let opt = self.room_objects_by_type.room_objects.get(id);
        let Some(r) = opt else {
            panic!("unable to get room object with id={:?}", *id);
        };
        r.room_obj.clone()
    }

    #[inline(never)]
    pub fn validate_end_tick(&self) {
        for v in self.room_objects_by_type.room_objects.values() {
            let ref_count = Rc::strong_count(&v.room_obj);
            // Player Rc strongs:
            // -One as field in r3run
            // -One as field in the current floor
            // -One in the RoomObjectsByType map of the room the player is in
            // -One as a local field in the calling function (run_floor_tick())
            if v.room_obj.borrow().is_player() {
                // might not be worth the overhead of validating the player's Rc strong count, because that requires
                // knowing all owners of Player, which is tough
                /*if ref_count != 4 {
                    panic!("RoomObject Player Rc::strong_count()={}. Expected 4. Id={:?}", ref_count, v.borrow().get_metadata().get_ref());
                }*/
            } else if ref_count != 1 {
                panic!("RoomObject Rc::strong_count()={}. Expected 1. Id={:?}", ref_count, v.room_obj.borrow().get_metadata().get_ref());
            }
        }
    }
}

pub struct NewRoomObjectContext<'a> {
    rofiz: &'a mut RofizState,
    room_object_id_counter: &'a mut RoomObjectId,
    room_time: f64,
    rng: &'a mut Prng,
}

impl<'a> NewRoomObjectContext<'a> {
    pub fn new(rofiz: &'a mut RofizState, room_object_id_counter: &'a mut RoomObjectId, room_time: f64, rng: &'a mut Prng) -> Self {
        Self {
            rofiz,
            room_object_id_counter,
            room_time,
            rng,
        }
    }

    pub fn from_gfr_ctx(rofiz: &'a mut RofizState, gfr_ctx: &'a mut GenFloorRoomContext) -> Self {
        Self {
            rofiz,
            room_object_id_counter: gfr_ctx.room_object_id_counter,
            room_time: 0.0,
            rng: gfr_ctx.rng,
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

    pub fn from_hc_ctx(hc_ctx: &'a mut HandleCollisionContext) -> Self {
        Self {
            rofiz: hc_ctx.rofiz,
            room_object_id_counter: hc_ctx.room_object_id_counter,
            room_time: hc_ctx.room_time,
            rng: hc_ctx.rng,
        }
    }

    pub fn get_and_inc_next_room_object_id(&mut self) -> RoomObjectId {
        *self.room_object_id_counter += 1;
        *self.room_object_id_counter
    }

    pub fn get_room_time(&mut self) -> f64 {
        self.room_time
    }

    pub fn get_rng(&mut self) -> &mut Prng {
        self.rng
    }

    // in the range [0, 1)
    pub fn get_randf64(&mut self) -> f64 {
        self.rng.gen_f64()
    }

    pub fn _get_randu64(&mut self, r: Range<u64>) -> u64 {
        self.rng.gen_u64_range(r)
    }

    pub fn get_randi64(&mut self, r: Range<i64>) -> i64 {
        self.rng.gen_i64_range(r)
    }

    pub fn add_basic_wall(&mut self, room_obj_ref: RoomObjectRef, x: u32, y: u32) -> RofizObjectRef {
        self.rofiz.add_basic_wall(room_obj_ref, x, y)
    }

    pub fn add_nonspectral_unit(&mut self, room_obj_ref: RoomObjectRef, hitbox: Hitbox) -> RofizObjectRef {
        self.rofiz.add_nonspectral_unit(room_obj_ref, hitbox)
    }

    pub fn add_spectral_unit(&mut self, room_obj_ref: RoomObjectRef, hitbox: Hitbox) -> RofizObjectRef {
        self.rofiz.add_spectral_unit(room_obj_ref, hitbox)
    }

    pub fn add_basic_projectile(&mut self, room_obj_ref: RoomObjectRef, hitbox: Hitbox) -> RofizObjectRef {
        self.rofiz.add_basic_projectile(room_obj_ref, hitbox)
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
    rng: &'a mut Prng,
    room_cleared_at_time: Option<f64>,
    sound_db: &'a SoundDb,
    sound_id_counter: &'a mut RoomObjSoundIdT,
    _room_width: u32,
    _room_height: u32,
    _room_tiles: &'a Vec<Vec<RoomTile>>,
}

impl<'a> Act1Context<'a> {
    pub fn new(
        player_input: &'a PlayerInput, 
        rofiz: &'a mut RofizState, 
        room_object_id_counter: &'a mut RoomObjectId, 
        tick_length: f64, 
        room_time: f64,
        rng: &'a mut Prng,
        room_cleared_at_time: Option<f64>,
        sound_db: &'a SoundDb,
        sound_id_counter: &'a mut RoomObjSoundIdT,
        room_width: u32,
        room_height: u32,
        room_tiles: &'a Vec<Vec<RoomTile>>,
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
            sound_db,
            sound_id_counter,
            _room_width: room_width,
            _room_height: room_height,
            _room_tiles: room_tiles,
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

    pub fn get_rng(&mut self) -> &mut Prng {
        self.rng
    }

    // in the range [0, 1)
    pub fn get_randf64(&mut self) -> f64 {
        self.rng.gen_f64()
    }

    pub fn get_randi64(&mut self, r: Range<i64>) -> i64 {
        self.rng.gen_i64_range(r)
    }

    pub fn get_room_cleared_at_time(&self) -> Option<f64> {
        self.room_cleared_at_time
    }

    pub fn is_in_room(&self, x: f64, y: f64) -> bool {
        x >= 0.0 && y >= 0.0 && x + 1.0 < self._room_width as f64 && y + 1.0 < self._room_height as f64
    }

    pub fn sample_wall_xy(&mut self, buffer: f64) -> (f64, f64) {
        let mut tries = 0;
        loop {
            tries += 1;
            assert!(tries < 100);
            let mut walk_len = self.rng.gen_f64() * (2.0 * (self._room_width + self._room_height) as f64);
            let (x, y) = (|| {
                if walk_len < self._room_width as f64 {
                    return (walk_len, -buffer);
                }
                walk_len -= self._room_width as f64;
                if walk_len < self._room_height as f64  {
                    return (self._room_width as f64 + buffer, walk_len);
                }
                walk_len -= self._room_height as f64;
                if walk_len < self._room_width as f64 {
                    return (walk_len, self._room_height as f64 + buffer);
                }
                walk_len -= self._room_width as f64;
                return (-buffer, walk_len);
            })();
            let xmin = i32::max(0, (x - buffer) as i32);
            let xmax = i32::min((self._room_width - 1).try_into().unwrap(), (x + buffer) as i32);
            let ymin = i32::max(0, (y - buffer) as i32);
            let ymax = i32::min((self._room_height - 1).try_into().unwrap(), (y + buffer) as i32);

            let mut works = true;
            'outer: for x in xmin..=xmax {
                for y in ymin..=ymax {
                    if self._room_tiles[x as usize][y as usize] == RoomTile::Connection {
                        works = false;
                        break 'outer;
                    }
                }
            }
            if works {
                return (x, y);
            }
        }
    }

    pub fn get_sound_db(&self) -> &SoundDb {
        self.sound_db
    }

    pub fn new_play_sound_builder(&mut self, sound_data: SoundDataRef) -> RoomObjPlaySoundArgsBuilder {
        new_play_sound_builder(self.sound_id_counter, sound_data)
    }
    
    pub fn _get_room_width(&self) -> u32 {
        self._room_width
    }

    pub fn _get_room_height(&self) -> u32 {
        self._room_height
    }

    pub fn _get_room_tiles(&self) -> &Vec<Vec<RoomTile>> {
        self._room_tiles
    }
}

pub fn new_play_sound_builder(id_counter: &mut RoomObjSoundIdT, sound_data: SoundDataRef) -> RoomObjPlaySoundArgsBuilder {
    *id_counter += 1;
    RoomObjPlaySoundArgsBuilder::new(RoomObjPlaySoundArgsBuilderReq {
        id: *id_counter,
        sound_data,
    })
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
    objects_to_remove: Vec<RoomObjectRef>,
    objects_to_add: Vec<Rc<RefCell<dyn RoomObject>>>,
    act1_queries: Vec<(Act1QueryArgs, Rc<RefCell<Act1QueryResult>>)>,
    operations: Vec<RoomObjOperation>,
    floor_finished: bool,
    owned_sound_playback_speed_override: Option<f64>,
    newly_played_sounds: Vec<RoomObjPlaySoundArgs>,
}

impl Act1Response {
    pub fn new() -> Self {
        Self {
            objects_to_remove: Vec::new(),
            objects_to_add: Vec::new(),
            act1_queries: Vec::new(),
            operations: Vec::new(),
            floor_finished: false,
            owned_sound_playback_speed_override: None,
            newly_played_sounds: Vec::new(),
        }
    }

    pub fn remove_room_obj(mut self, r: RoomObjectRef) -> Self {
        self.objects_to_remove.push(r);
        self
    }

    pub fn steal_room_objs_to_remove(&mut self, v: &mut Vec<RoomObjectRef>) {
        v.append(&mut self.objects_to_remove);
    }

    pub fn add_room_obj(&mut self, obj: Rc<RefCell<dyn RoomObject>>) {
        self.objects_to_add.push(obj);
    }

    pub fn steal_room_objs_to_add(&mut self, v: &mut Vec<Rc<RefCell<dyn RoomObject>>>) {
        v.append(&mut self.objects_to_add);
    }

    pub fn add_query(&mut self, args: Act1QueryArgs) -> Rc<RefCell<Act1QueryResult>> {
        let rc_result = Rc::new(RefCell::new(Act1QueryResult::NotSet));
        self.act1_queries.push((args, rc_result.clone()));
        rc_result
    }

    pub fn steal_queries(&mut self, v: &mut Vec<(Act1QueryArgs, Rc<RefCell<Act1QueryResult>>)>) {
        v.append(&mut self.act1_queries);
    }

    pub fn apply_operation(&mut self, op: RoomObjOperation) {
        self.operations.push(op);
    }

    pub fn steal_operations(&mut self, v: &mut Vec<RoomObjOperation>) {
        v.append(&mut self.operations);
    }

    pub fn finish_floor(&mut self) {
        self.floor_finished = true;
    }

    pub fn get_is_floor_finished(&self) -> bool {
        self.floor_finished
    }

    pub fn play_sound(&mut self, args: RoomObjPlaySoundArgs) -> RoomObjSoundRef {
        let id = args.id;
        self.newly_played_sounds.push(args);
        RoomObjSoundRef::new(id)
    }

    pub fn take_owned_sound_playback_speed_override(&mut self) -> Option<f64> {
        self.owned_sound_playback_speed_override.take()
    }

    pub fn take_newly_played_sounds(&mut self) -> Vec<RoomObjPlaySoundArgs> {
        std::mem::take(&mut self.newly_played_sounds)
    }
}

pub struct HandleRoomJustClearedContext<'a> {
    _rofiz: &'a mut RofizState,
    room_time: f64,
    starcash_reward: f64,
}

impl<'a> HandleRoomJustClearedContext<'a> {
    pub fn _get_rofiz(&mut self) -> &mut RofizState {
        self._rofiz
    }

    pub fn get_room_time(&self) -> f64 {
        self.room_time
    }

    pub fn get_starcash_reward(&self) -> f64 {
        self.starcash_reward
    }
}

pub struct HandleCollisionContext<'a> {
    other: Rc<RefCell<dyn RoomObject>>,
    other_is_spectral: bool,
    rng: &'a mut Prng,
    room_object_id_counter: &'a mut RoomObjectId,
    room_time: f64,
    self_as_rc: Rc<RefCell<dyn RoomObject>>,
    _tick_length: f64,
    rofiz: &'a mut RofizState,
}

impl<'a> HandleCollisionContext<'a> {
    pub fn new(
        other: Rc<RefCell<dyn RoomObject>>, 
        other_is_spectral: bool, 
        rng: &'a mut Prng, 
        room_object_id_counter: &'a mut RoomObjectId,
        room_time: f64, 
        self_as_rc: Rc<RefCell<dyn RoomObject>>,
        tick_length: f64, 
        rofiz: &'a mut RofizState,
    ) -> Self {
        Self { 
            other,
            other_is_spectral,
            rng,
            room_object_id_counter,
            room_time,
            self_as_rc,
            _tick_length: tick_length,
            rofiz,
        }
    }
    
    pub fn get_other(&mut self) -> Rc<RefCell<dyn RoomObject>> {
        self.other.clone()
    }

    pub fn is_other_spectral(&self) -> bool {
        self.other_is_spectral
    }

    // in the range [0, 1)
    pub fn _get_randf64(&mut self) -> f64 {
        self.rng.gen_f64()
    }

    pub fn _get_randu64(&mut self, r: Range<u64>) -> u64 {
        self.rng.gen_u64_range(r)
    }

    pub fn get_randi64(&mut self, r: Range<i64>) -> i64 {
        self.rng.gen_i64_range(r)
    }

    pub fn get_room_time(&self) -> f64 {
        self.room_time
    }

    pub fn get_self_as_weak(&self) -> Weak<RefCell<dyn RoomObject>> {
        Rc::downgrade(&self.self_as_rc)
    }

    pub fn get_tick_length(&self) -> f64 {
        self._tick_length
    }

    pub fn get_rofiz(&self) -> &RofizState {
        self.rofiz
    }
}

pub struct HandleCollisionResponse {
    room_objects_to_remove: Vec<RoomObjectRef>,
    room_objects_to_add: Vec<Rc<RefCell<dyn RoomObject>>>,
}

impl HandleCollisionResponse {
    pub fn new() -> Self {
        Self { 
            room_objects_to_remove: Vec::new(),
            room_objects_to_add: Vec::new(),
        }
    }

    pub fn remove_room_obj(mut self, id: RoomObjectRef) -> Self {
        self.room_objects_to_remove.push(id);
        self
    }

    pub fn remove_room_objs(mut self, ids: &[RoomObjectRef]) -> Self {
        self.room_objects_to_remove.extend(ids);
        self
    }

    pub fn get_room_objects_to_remove(&self) -> &[RoomObjectRef] {
        self.room_objects_to_remove.as_slice()
    }

    pub fn add_room_obj(mut self, room_obj: Rc<RefCell<dyn RoomObject>>) -> Self {
        self.room_objects_to_add.push(room_obj);
        self
    }

    pub fn steal_room_objs_to_add(&mut self) -> Vec<Rc<RefCell<dyn RoomObject>>> {
        let mut v = Vec::new();
        std::mem::swap(&mut self.room_objects_to_add, &mut v);
        v
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Team {
    Player,
    Enemy,
}

impl Team {
    pub fn other(&self) -> Self {
        match self {
            Team::Player => Team::Enemy,
            Team::Enemy => Team::Player,
        }
    }
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
    pub room_objects_to_delete: Vec<RoomObjectRef>,
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

pub struct HcStandardUnitContext<'a> {
    pub suc: &'a mut StandardUnitCommon,
    pub team: Team,
    pub damage_color: DamageColor,
}

pub struct HcStandardUnitResponse {
    pub room_objects_to_delete: Vec<RoomObjectRef>,
}

pub struct HcBlackHoleContext {
    pub affects_projectiles_color_filter: Option<DamageColor>,
}

pub struct HcBlackHoleResponse {
    pub room_objects_to_delete: Vec<RoomObjectRef>,
}

pub struct HcTileContext {
    pub tile_effect: HcTileEffect,
}

#[derive(Debug, Clone, Copy)]
pub enum HcTileEffect {
    Accelerate {force: f64, theta: f64},
    DealDamage {damage: f64},
    TractionMult {mult: f64, duration: HcTileEffectDuration},
    TractionCap {cap: f64, duration: HcTileEffectDuration},
    ChargeTile {},
}

#[derive(Debug, Clone, Copy)]
pub enum HcTileEffectDuration {
    OneTick,
    Time(f64)
}

pub struct HcTileResponse {
    pub unit_affected: bool,
}