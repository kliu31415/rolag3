use crate::rolag3::floor::{draw::DrawContext, run::PlayerInput, rofiz::{rofiz_state::RofizState, rofiz_object::{Hitbox, RofizObjectRef}}};

pub trait FloorObject {
    fn get_id(&self) -> FloorObjectId;
    fn act1(&mut self, ctx: &mut Act1Context);
    fn draw(&self, ctx: &mut DrawContext);

    fn is_wall_like(&self) -> bool {
        false
    }
    fn is_projectile_like(&self) -> bool {
        false
    }
}

pub type FloorObjectId = usize;
pub struct NewFloorObjectContext<'a> {
    current_floor_object_id_counter: FloorObjectId,
    rofiz: &'a mut RofizState,
}

impl<'a> NewFloorObjectContext<'a> {
    pub fn new(rofiz: &'a mut RofizState) -> Self {
        Self {
            current_floor_object_id_counter: 0,
            rofiz,
        }
    }
    pub fn get_next_floor_object_id(&mut self) -> FloorObjectId {
        self.current_floor_object_id_counter += 1;
        self.current_floor_object_id_counter
    }

    pub fn add_basic_wall(&mut self, floor_object_id: FloorObjectId, x: u32, y: u32) -> RofizObjectRef {
        self.rofiz.add_basic_wall(floor_object_id, x, y)
    }

    pub fn add_nonspectral_unit(&mut self, floor_object_id: FloorObjectId, hitbox: Hitbox) -> RofizObjectRef {
        self.rofiz.add_nonspectral_unit(floor_object_id, hitbox)
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
}

impl<'a> Act1Context<'a> {
    pub fn new(player_input: &'a PlayerInput, rofiz: &'a mut RofizState) -> Self {
        Self {
            player_input,
            rofiz,
        }
    }
    pub fn get_player_input(&self) -> &PlayerInput {
        self.player_input
    }
    pub fn get_rofiz(&mut self) -> &mut RofizState {
        self.rofiz
    }
}

/*
struct BasicGround {

}

struct BasicProjectile {

}
*/