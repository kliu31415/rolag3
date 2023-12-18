use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{projectile::projectile2::{Proj2Shape, NewProjectile2Args}, damage::DamageColor}, draw::Color}, geometry::shape::Point, gfx::{draw_op_util::draw_op_rect, renderer::{ColorRGBA32f, DrawOp, DrawOpGroup}}};

use super::weapon_def::{WeaponHandleTickContext, Weapon, WeaponHandleTickResponse, DrawWeaponHudContext, DrawWeaponHudResponse};

/* Weapon1 rapidly shoots green squares, like a laser. It has no special attack. */

const PRIMARY_ATTACK_INTERVAL: f64 = 0.003;
const PROJ_COLOR: Color = Color::new(0.0, 1.6, 0.0, 1.0);

struct Weapon1Data {
    since_last_primary_attack: f64,
}

pub fn new_weapon1() -> Weapon {
    let ws_data = Box::new(Weapon1Data {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
    });
    Weapon::new(ws_data, Box::new(handle_tick_fn), Box::new(draw_hud))
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<Weapon1Data>().unwrap();
    let mut response = WeaponHandleTickResponse::new();

    ws_data.since_last_primary_attack += ctx.tick_len;
    ws_data.since_last_primary_attack = f64::min(ws_data.since_last_primary_attack, 1.5 * PRIMARY_ATTACK_INTERVAL);
    if !ctx.primary_attack {
        return response;
    }
    if ws_data.since_last_primary_attack < PRIMARY_ATTACK_INTERVAL {
        return response;
    }
    ws_data.since_last_primary_attack -= PRIMARY_ATTACK_INTERVAL;

    let proj_velocity = 100.0;
    let proj_angle = f64::atan2(ctx.mouse_y - ctx.owner_xform.dy, ctx.mouse_x - ctx.owner_xform.dx);
    let velocity_x = ctx.owner_velocity_x + proj_velocity * f64::cos(proj_angle);
    let velocity_y = ctx.owner_velocity_y + proj_velocity * f64::sin(proj_angle);
    let shape = Proj2Shape::TriFan {
        center: Point::new(0.0, 0.0), 
        vertexes: vec![Point::new(-0.2, -0.2), Point::new(0.2, -0.2), Point::new(0.2, 0.2), Point::new(-0.2, 0.2)].into_boxed_slice() 
    };
    let proj = NewProjectile2Args {
        team: ctx.owner_team,
        damage_color: DamageColor::Green,
        damage: 2.0,
        owner: ctx.owner.clone(),
        lifespan: 2.0,
        velocity_x,
        velocity_y,
        xform: ctx.owner_xform,
        shape,
        color: PROJ_COLOR,
    }.new(ctx.nro_ctx);
    
    response.new_room_objs.push(Rc::new(RefCell::new(proj)));
    response
}

fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let background_color = if ctx.is_selected {
        ColorRGBA32f::new(1.0, 1.0, 1.0, 0.2)
    } else {
        ColorRGBA32f::new(0.8, 0.8, 0.8, 0.1)
    };
    let background = draw_op_rect(background_color, ctx.x, ctx.y, ctx.scale_height * 3.0, ctx.scale_height);
    let buffer_px = 0.1 * ctx.scale_height;
    let inner_scale = 0.8 * ctx.scale_height;
    let draw_op = draw_op_rect((&PROJ_COLOR).into(), ctx.x + buffer_px, ctx.y + buffer_px, inner_scale, inner_scale);
    DrawWeaponHudResponse { draw_op: DrawOp::Group(DrawOpGroup::new(vec![background, draw_op].into_boxed_slice())) }
}