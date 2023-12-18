use std::{rc::{Rc, Weak}, cell::RefCell, any::Any};

use crate::rolag3::floor::{room_object::room_object_def::{RoomObject, NewRoomObjectContext, Team}, rofiz::rofiz_object::Transformation};

type WeaponHandleTickFn = dyn Fn(&mut WeaponHandleTickContext) -> WeaponHandleTickResponse;

pub struct Weapon {
    pub ws_data: Box<dyn Any>,
    pub handle_tick_fn: Box<WeaponHandleTickFn>,
}

impl Weapon {
    pub fn new(ws_data: Box<dyn Any>, handle_tick_fn: Box<WeaponHandleTickFn>) -> Self {
        Self {
            ws_data,
            handle_tick_fn,
        }
    }
}

pub struct WeaponHandleTickContext<'a> {
    pub ws_data: &'a mut dyn Any,
    pub tick_len: f64,
    pub nro_ctx: &'a mut NewRoomObjectContext<'a>,
    pub owner: Weak<RefCell<dyn RoomObject>>,
    pub owner_team: Team,
    pub owner_velocity_x: f64,
    pub owner_velocity_y: f64,
    pub owner_xform: Transformation,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub primary_attack: bool,
    pub special_attack: bool,
    pub owner_mana: f64,
}

pub struct WeaponHandleTickResponse {
    pub new_room_objs: Vec<Rc<RefCell<dyn RoomObject>>>,
    pub mana_delta: f64,
}

impl WeaponHandleTickResponse {
    pub fn new() -> Self {
        WeaponHandleTickResponse { new_room_objs: Vec::new(), mana_delta: 0.0 }
    }
}