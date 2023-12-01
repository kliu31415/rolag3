use std::{rc::{Rc, Weak}, cell::{RefCell, Ref}, collections::HashSet};

use rand::{rngs::ThreadRng, Rng};

use crate::rolag3::floor::{draw::DrawContext, run::PlayerInput, rofiz::{rofiz_state::{RofizState, RofizObjectRef}, rofiz_object::Hitbox}};

pub trait RoomObject {
    fn get_metadata(&self) -> &RoomObjectMetadata;
    fn act1(&mut self, ctx: &mut Act1Context) -> Act1Response;
    fn draw(&self, ctx: &mut DrawContext);

    fn handle_collision(&mut self, ctx: &mut HandleCollisionContext) -> HandleCollisionResponse;

    fn is_wall_like(&self) -> bool {
        false
    }
    fn is_projectile_like(&self) -> bool {
        false
    }
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
    pub fn get_id(&self) -> RoomObjectId {
        self.id
    }
}

pub struct RoomObjectCollection {
    objects: Vec<Rc<RefCell<dyn RoomObject>>>,
}

impl RoomObjectCollection {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: Rc<RefCell<dyn RoomObject>>) {
        self.objects.push(obj);
    }

    pub fn _remove(&mut self, id: RoomObjectId) {
        let current_count = self.objects.len();
        self.objects.retain(|x| x.borrow().get_metadata().get_id() != id);
        let new_count = self.objects.len();
        if new_count + 1 != current_count {
            panic!("expected to remove 1 RoomObject, but didn't. Prev count={}, new count = {}", current_count, new_count);
        }
    }

    pub fn _count(&self) -> usize {
        self.objects.len()
    }

    pub fn act1(&mut self, ctx: &mut Act1Context) {
        let mut responses = Vec::new();
        self.objects.iter().for_each(|x| {
            ctx.set_self_as_weak(Rc::downgrade(x));
            responses.push(x.borrow_mut().act1(ctx))}
        );
        let should_remove: Vec<bool> = responses.iter().map(|x| x.get_remove_me()).collect();
        self.remove_all_using_bool_array(should_remove.as_slice());

        responses.iter_mut().flat_map(|x| x.steal_room_objs_to_add()).for_each(|x| self.objects.push(x));
    }

    pub fn remove_by_id(&mut self, to_remove: HashSet<RoomObjectId>) {
        self.objects.retain(|x| !to_remove.contains(&x.borrow().get_metadata().get_id()));
    }

    pub fn draw(&mut self, ctx: &mut DrawContext) {
        for fo in self.objects.iter() {
            fo.borrow_mut().draw(ctx);
        }
    }

    pub fn _apply(&self, f: &mut dyn FnMut(&Ref<dyn RoomObject>)) {
        let iter = self.objects.iter().map(|x| x.borrow());
        for fo in iter {
            f(&fo);
        }
    }

    pub fn remove_all_using_bool_array(&mut self, should_remove: &[bool]) {
        let obj_count = self.objects.len();
        if should_remove.len() != obj_count {
            panic!("should_remove.len() != obj_count. Values: {} != {}", should_remove.len(), obj_count)
        }
        let mut idx = 0;
        self.objects.retain(|_| {idx += 1; !should_remove[idx - 1]});
    }

    pub fn get(&self, id: RoomObjectId) -> Rc<RefCell<dyn RoomObject>> {
        let iter = self.objects.iter();
        for fo in iter {
            if id == fo.borrow().get_metadata().get_id() {
                return fo.clone();
            }
        }
        panic!("unable to find RoomObject with id={}", id)
    }
}

pub struct NewRoomObjectContext<'a> {
    rofiz: &'a mut RofizState,
    room_object_id_counter: &'a mut RoomObjectId,
}

impl<'a> NewRoomObjectContext<'a> {
    pub fn new(rofiz: &'a mut RofizState, room_object_id_counter: &'a mut RoomObjectId) -> Self {
        Self {
            rofiz,
            room_object_id_counter,
        }
    }

    pub fn from_act1_ctx(act1_ctx: &'a mut Act1Context) -> Self {
        Self {
            rofiz: act1_ctx.rofiz,
            room_object_id_counter: act1_ctx.room_object_id_counter,
        }
    }

    pub fn get_next_floor_object_id(&mut self) -> RoomObjectId {
        *self.room_object_id_counter += 1;
        *self.room_object_id_counter
    }

    pub fn add_basic_wall(&mut self, floor_object_id: RoomObjectId, x: u32, y: u32) -> RofizObjectRef {
        self.rofiz.add_basic_wall(floor_object_id, x, y)
    }

    pub fn add_nonspectral_unit(&mut self, floor_object_id: RoomObjectId, hitbox: Hitbox) -> RofizObjectRef {
        self.rofiz.add_nonspectral_unit(floor_object_id, hitbox)
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
    self_as_weak: Option<Weak<RefCell<dyn RoomObject>>>,
    tick_length: f64,
    rng: &'a mut ThreadRng,
}

impl<'a> Act1Context<'a> {
    pub fn new(player_input: &'a PlayerInput, rofiz: &'a mut RofizState, room_object_id_counter: &'a mut RoomObjectId, tick_length: f64, rng: &'a mut ThreadRng) -> Self {
        Self {
            player_input,
            rofiz,
            room_object_id_counter,
            self_as_weak: Option::None,
            tick_length,
            rng,
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

    pub fn set_self_as_weak(&mut self, weak: Weak<RefCell<dyn RoomObject>>) {
        self.self_as_weak = Some(weak);
    }

    pub fn self_as_weak(&self) -> Weak<RefCell<dyn RoomObject>> {
        self.self_as_weak.clone().unwrap()
    }

    // in the range [0, 1)
    pub fn get_randf64(&mut self) -> f64 {
        self.rng.gen::<f64>()
    }
} 

pub struct Act1Response {
    should_remove_me: bool,
    objects_to_add: Vec<Rc<RefCell<dyn RoomObject>>>,
}

impl Act1Response {
    pub fn new() -> Self {
        Self {
            should_remove_me: false,
            objects_to_add: Vec::new(),
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
}

pub struct HandleCollisionContext<'a> {
    other: Rc<RefCell<dyn RoomObject>>,
    rng: &'a mut ThreadRng,
}

impl<'a> HandleCollisionContext<'a> {
    pub fn new(other: Rc<RefCell<dyn RoomObject>>, rng: &'a mut ThreadRng) -> Self {
        Self { 
            other,
            rng,
        }
    }
    
    pub fn get_other(&self) -> Rc<RefCell<dyn RoomObject>> {
        self.other.clone()
    }

    // in the range [0, 1)
    pub fn get_randf64(&mut self) -> f64 {
        self.rng.gen::<f64>()
    }
}

pub struct HandleCollisionResponse {
    should_remove_me: bool,
}

impl HandleCollisionResponse {
    pub fn new() -> Self {
        Self { 
            should_remove_me: false,
        }
    }

    pub fn remove_me(mut self) -> Self {
        self.should_remove_me = true;
        self
    }

    pub fn get_remove_me(&self) -> bool {
        self.should_remove_me
    }
}

/*
struct BasicGround {

}

struct BasicProjectile {

}
*/