use crate::rolag3::floor::{draw::DrawContext, run::PlayerInput, rofiz::{rofiz_state::RofizState, rofiz_object::{Hitbox, RofizObjectRef}}};

use super::unit::player::Player;

pub trait RoomObject {
    fn get_metadata(&self) -> &RoomObjectMetadata;
    fn act1<'a, 'b>(&mut self, ctx: &'a mut Act1Context<'b>) -> Act1Response;
    fn draw(&self, ctx: &mut DrawContext);

    fn handle_collision(&mut self, other: &dyn RoomObject);

    fn is_wall_like(&self) -> bool {
        false
    }
    fn is_projectile_like(&self) -> bool {
        false
    }
}

pub struct RoomObjectMetadata {
    id: RoomObjectId,
    child_objects: RoomObjectCollection,
}

pub type RoomObjectId = usize;

impl RoomObjectMetadata {
    pub fn new(ctx: &mut NewRoomObjectContext) -> Self {
        Self { 
            id: ctx.get_next_floor_object_id(),
            child_objects: RoomObjectCollection::new(),
        }
    }
    pub fn get_id(&self) -> RoomObjectId {
        self.id
    }
    pub fn get_child_objects_mut(&mut self) -> &mut RoomObjectCollection {
        &mut self.child_objects
    }
    pub fn get_child_objects(&self) -> &RoomObjectCollection {
        &self.child_objects
    }
}

pub struct RoomObjectCollection {
    player: Option<Box<Player>>,
    objects: Vec<Box<dyn RoomObject>>,
}

impl RoomObjectCollection {
    pub fn new() -> Self {
        Self {
            player: None,
            objects: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: Box<dyn RoomObject>) {
        self.objects.push(obj);
    }

    pub fn remove(&mut self, id: RoomObjectId) {
        let current_count = self.objects.len();
        self.objects.retain(|x| x.get_metadata().get_id() != id);
        let new_count = self.objects.len();
        if new_count + 1 != current_count {
            panic!("expected to remove 1 FloorObject, but didn't. Prev count={}, new count = {}", current_count, new_count);
        }
    }

    pub fn count(&self) -> usize {
        // todo: recursively count children
        self.objects.len() + self.player.iter().len()
    }

    pub fn set_player(&mut self, player: Box<Player>) {
        self.player = Some(player);
    }

    pub fn get_player(&self) -> &Player {
        self.player.as_ref().unwrap()
    }

    pub fn _steal_player(&mut self) -> Box<Player> {
        self.player.take().unwrap()
    }

    // all iteration logic should put the player last, after all room_objects. The function remove_by_index() assumes
    // that apply_mut() does this.
    pub fn apply_mut(&mut self, f: &mut dyn FnMut(&mut dyn RoomObject)) {
        let iter1 = self.objects.iter_mut().map(|x| x.as_mut());
        let iter2 = self.player.iter_mut().map(|x| x.as_mut() as &mut dyn RoomObject);
        for fo in iter1.chain(iter2) {
            f(fo);
        }
    }

    pub fn apply(&self, f: &mut dyn FnMut(&dyn RoomObject)) {
        let iter1 = self.objects.iter().map(|x| x.as_ref());
        let iter2 = self.player.iter().map(|x| x.as_ref() as &dyn RoomObject);
        for fo in iter1.chain(iter2) {
            f(fo);
        }
    }

    pub fn remove_all_using_bool_array(&mut self, should_remove: &[bool]) {
        let obj_count = self.objects.len() + self.player.iter().len();
        if should_remove.len() != obj_count {
            panic!("should_remove.len() != obj_count. Values: {} != {} + {}", should_remove.len(), self.objects.len(), self.player.iter().len())
        }
        if should_remove.is_empty() {
            return;
        }
        if self.player.is_some() && *should_remove.last().unwrap() {
            panic!("can't remove player from FloorObjectCollection");
        }
        let mut idx = 0;
        self.objects.retain(|x| {idx += 1; !should_remove[idx - 1]});
    }

    pub fn get_two_floor_obj_mut(&mut self, id1: RoomObjectId, id2: RoomObjectId) -> (&mut dyn RoomObject, &mut dyn RoomObject){
        // TODO: optimize this. Right now, it's O(n)
        if id1 == id2 {
            panic!("unable to get two mutable references to the same FloorObject. id1 and id2 are the same, id={}", id1);
        }
        let mut fo1 = Option::None;
        let mut fo2 = Option::None;
        let iter1 = self.objects.iter_mut().map(|x| x.as_mut());
        let iter2 = self.player.iter_mut().map(|x| x.as_mut() as &mut dyn RoomObject);
        for fo in iter1.chain(iter2) {
            if fo.get_metadata().get_id() == id1 {
                if fo1.is_some() {
                    panic!("found multiple FloorObjects matching id1={}", id1);
                }
                fo1 = Option::Some(fo);
            } else if fo.get_metadata().get_id() == id2 {
                if fo2.is_some() {
                    panic!("found multiple FloorObjects matching id2={}", id1);
                }
                fo2 = Option::Some(fo);
            }
        }

        if fo1.is_none() {
            panic!("unable to find FloorObject 1, with id1={}", id1);
        }
        if fo2.is_none() {
            panic!("unable to find FloorObject 2, with id2={}", id2);
        }
        (fo1.unwrap(), fo2.unwrap())
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
    tick_length: f64,
}

impl<'a> Act1Context<'a> {
    pub fn new(player_input: &'a PlayerInput, rofiz: &'a mut RofizState, room_object_id_counter: &'a mut RoomObjectId, tick_length: f64) -> Self {
        Self {
            player_input,
            rofiz,
            room_object_id_counter,
            tick_length,
        }
    }

    pub fn get_player_input(&self) -> &PlayerInput {
        self.player_input
    }

    pub fn get_rofiz(&mut self) -> &mut RofizState {
        self.rofiz
    }

    pub fn get_room_object_id_counter(&'a mut self) -> &'a mut RoomObjectId {
        self.room_object_id_counter
    }

    pub fn get_tick_length(&self) -> f64 {
        self.tick_length
    }
} 

pub struct Act1Response {
    should_remove_me: bool,
}

impl Act1Response {
    pub fn new() -> Self {
        Self {
            should_remove_me: false
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