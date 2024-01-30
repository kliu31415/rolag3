use std::{rc::{Rc, Weak}, cell::RefCell, any::Any};

use crate::{rolag3::{floor::{room_object::{room_object_def::{RoomObject, NewRoomObjectContext, Team}, damage::DamageColor, sound::{RoomObjPlaySoundArgs, RoomObjSoundIdT}}, rofiz::rofiz_object::Transformation, draw::{DrawContext, Color}}, sound_db::SoundDb}, gfx::{renderer::{DrawOp, DrawOpText, DrawTextPosition, DrawOpGroup, ColorRGBA32f}, draw_op_util::draw_op_rect, text::font::Font}};

type WeaponHandleTickFn = dyn Fn(&mut WeaponHandleTickContext) -> WeaponHandleTickResponse;
type DrawWeaponHudFn = dyn Fn(&DrawWeaponHudContext) -> DrawWeaponHudResponse;
type DrawWeaponOnOwnerFn = dyn Fn(&DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse;

pub struct Weapon {
    pub damage_color: DamageColor,
    pub name: &'static str,
    pub shop_description: &'static str,
    pub shop_cost: u64,
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
        damage_color: DamageColor,
        name: &'static str,
        shop_description: &'static str,
        shop_cost: u64,
        ammo: f64,
        buy_ammo_info: Option<BuyAmmoInfo>,
        ws_data: Box<dyn Any>, 
        handle_tick_fn: Box<WeaponHandleTickFn>, 
        draw_hud_fn: Box<DrawWeaponHudFn>, 
        draw_on_owner_fn: Box<DrawWeaponOnOwnerFn>,
    ) -> Self {
        Self {
            damage_color,
            name,
            shop_description,
            shop_cost,
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
    pub owner_age: f64,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub primary_attack: bool,
    pub special_attack: bool,
    pub owner_mana: f64,
    pub sound_db: &'a SoundDb,
    pub sound_id_counter: &'a mut RoomObjSoundIdT,
}

pub struct WeaponHandleTickResponse {
    pub new_room_objs: Vec<Rc<RefCell<dyn RoomObject>>>,
    pub mana_delta: f64,
    pub damage_color: DamageColor,
    pub newly_played_sounds: Vec<RoomObjPlaySoundArgs>,
}

impl WeaponHandleTickResponse {
    pub fn new() -> Self {
        WeaponHandleTickResponse { 
            new_room_objs: Vec::new(), 
            mana_delta: 0.0, 
            damage_color: DamageColor::NotSet,
            newly_played_sounds: Vec::new(),
        }
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
    pub ws_data: &'a dyn Any,
    pub draw_ctx: &'a DrawContext<'a>,
    // (x, y) repesents the center of the owner
    pub x: f32,
    pub y: f32,

    pub owner_xform: Transformation,
    pub mouse_x_game_coords: f64,
    pub mouse_y_game_coords: f64,
}

pub struct DrawWeaponOnOwnerResponse {
    pub draw_op: DrawOp,
    pub owner_color: Color,
}

impl Weapon {
    pub fn draw(&self,
        background_color: ColorRGBA32f, 
        ammo_text_color: ColorRGBA32f,
        x: f32, 
        y: f32, 
        row_width: f32,
        row_height: f32, 
    ) -> DrawOp {
        let mut this_row_ops = Vec::new();
        let background = draw_op_rect(background_color, x, y, row_width, row_height);
        this_row_ops.push(background);
        let buffer_px = 0.15 * row_height;
        let inner_scale = row_height - 2.0 * buffer_px;
    
        let dwh_ctx = DrawWeaponHudContext { 
            scale_height: inner_scale,
            x: x + buffer_px,
            y: y + buffer_px,
            ammo: self.ammo,
        };
        let r = (self.draw_hud_fn)(&dwh_ctx);
        this_row_ops.push(r.weapon_draw_op);
    
        let ammo_text = DrawOp::Text(DrawOpText { 
            text: r.ammo_text.clone(), 
            font: Font::TekoRegular,
            color: ammo_text_color,
            x: x + row_height, 
            y, 
            font_size: row_height,
            position: DrawTextPosition::TopLeft,
        });
        this_row_ops.push(ammo_text);
    
        DrawOp::Group(DrawOpGroup { ops: this_row_ops.into_boxed_slice() })
    }
}