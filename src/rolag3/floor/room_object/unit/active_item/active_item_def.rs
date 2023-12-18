use std::any::Any;

use crate::rolag3::floor::room_object::room_object_def::{RoomObjOperation, Team};

pub type ActiveItemHandleTickFn = dyn Fn(&mut ActiveItemHandleTickContext) -> ActiveItemHandleTickResponse;

pub struct ActiveItem {
    pub ais_data: Box<dyn Any>,
    pub handle_tick_fn: Box<ActiveItemHandleTickFn>,
}

pub struct ActiveItemHandleTickContext<'a> {
    pub ais_data: &'a mut dyn Any,
    pub owner_team: Team,
    pub owner_x: f64,
    pub owner_y: f64,
    pub owner_mana: f64,
    pub tick_len: f64,
    pub use_this_item: bool,
}

pub struct ActiveItemHandleTickResponse {
    pub mana_delta: f64,
    pub ops: Vec<RoomObjOperation>,
}

impl ActiveItemHandleTickResponse {
    pub fn new() -> Self {
        ActiveItemHandleTickResponse { 
            mana_delta: 0.0,
            ops: Vec::new(),
        }
    }
}