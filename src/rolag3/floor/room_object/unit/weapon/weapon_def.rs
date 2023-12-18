use std::{rc::{Rc, Weak}, cell::RefCell, any::Any};

use crate::{rolag3::floor::{room_object::room_object_def::{RoomObject, NewRoomObjectContext, Team}, rofiz::rofiz_object::Transformation}, gfx::renderer::DrawOp};

type WeaponHandleTickFn = dyn Fn(&mut WeaponHandleTickContext) -> WeaponHandleTickResponse;
type DrawWeaponHudFn = dyn Fn(&DrawWeaponHudContext) -> DrawWeaponHudResponse;

pub struct Weapon {
    pub ws_data: Box<dyn Any>,
    pub handle_tick_fn: Box<WeaponHandleTickFn>,
    pub draw_hud_fn: Box<DrawWeaponHudFn>,
}

impl Weapon {
    pub fn new(ws_data: Box<dyn Any>, handle_tick_fn: Box<WeaponHandleTickFn>, draw_hud_fn: Box<DrawWeaponHudFn>) -> Self {
        Self {
            ws_data,
            handle_tick_fn,
            draw_hud_fn,
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

pub struct DrawWeaponHudContext {
    pub scale_height: f32,
    pub x: f32,
    pub y: f32,
    pub is_selected: bool,
}

pub struct DrawWeaponHudResponse {
    pub weapon_draw_op: DrawOp,
    pub ammo_text: String,
}