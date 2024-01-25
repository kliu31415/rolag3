use std::{rc::{Rc, Weak}, cell::RefCell, any::Any};

use crate::{rolag3::floor::{room_object::{room_object_def::{RoomObject, NewRoomObjectContext, Team}, damage::DamageColor}, rofiz::rofiz_object::Transformation, draw::{DrawContext, Color}}, gfx::renderer::DrawOp};

type WeaponHandleTickFn = dyn Fn(&mut WeaponHandleTickContext) -> WeaponHandleTickResponse;
type DrawWeaponHudFn = dyn Fn(&DrawWeaponHudContext) -> DrawWeaponHudResponse;
type DrawWeaponOnOwnerFn = dyn Fn(&DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse;

pub struct Weapon {
    pub name: &'static str,
    pub shop_description: &'static str,
    pub ammo: f64,
    pub buy_ammo_info: Option<BuyAmmoInfo>,
    pub ws_data: Box<dyn Any>,
    pub handle_tick_fn: Box<WeaponHandleTickFn>,
    pub draw_hud_fn: Box<DrawWeaponHudFn>,
    pub draw_on_owner_fn: Box<DrawWeaponOnOwnerFn>,
}

pub struct BuyAmmoInfo {
    pub ammo_amount: f64,
    pub starcash_cost: f64,
}

impl Weapon {
    pub fn new(
        name: &'static str,
        shop_description: &'static str,
        ammo: f64,
        buy_ammo_info: Option<BuyAmmoInfo>,
        ws_data: Box<dyn Any>, 
        handle_tick_fn: Box<WeaponHandleTickFn>, 
        draw_hud_fn: Box<DrawWeaponHudFn>, 
        draw_on_owner_fn: Box<DrawWeaponOnOwnerFn>,
    ) -> Self {
        Self {
            name,
            shop_description,
            ammo,
            buy_ammo_info,
            ws_data,
            handle_tick_fn,
            draw_hud_fn,
            draw_on_owner_fn,
        }
    }
}

pub struct WeaponHandleTickContext<'a> {
    pub ws_data: &'a mut dyn Any,
    pub ammo: &'a mut f64,
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
    pub damage_color: DamageColor,
}

impl WeaponHandleTickResponse {
    pub fn new() -> Self {
        WeaponHandleTickResponse { new_room_objs: Vec::new(), mana_delta: 0.0, damage_color: DamageColor::NotSet}
    }
}

pub struct DrawWeaponHudContext {
    // all fields represent measurements in pixels (VSC coordinates)
    pub scale_height: f32,
    // (x, y) represent the top left corner pixel coordinate
    pub x: f32,
    pub y: f32,

    pub ammo: f64,
}

pub struct DrawWeaponHudResponse {
    pub weapon_draw_op: DrawOp,
    pub ammo_text: String,
}

pub struct DrawWeaponOnOwnerContext<'a> {
    // all fields represent measurements in game map tiles (game units)
    pub draw_ctx: &'a DrawContext<'a>,
    // (x, y) repesents the center of the owner
    pub x: f32,
    pub y: f32,
}

pub struct DrawWeaponOnOwnerResponse {
    pub draw_op: DrawOp,
    pub owner_color: Color,
}
