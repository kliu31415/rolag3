use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{projectile::projectile2::{Proj2Shape, Projectile2BuilderReq, Projectile2Builder}, damage::DamageColor}, draw::Color}, geometry::{shape::{Point, Vector}, util::translate_polygon}, gfx::draw_op_util::draw_op_rect};

use super::weapon_def::{WeaponHandleTickContext, Weapon, WeaponHandleTickResponse, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerContext, DrawWeaponOnOwnerResponse};

/* Weapon1 rapidly shoots green squares, like a laser. It has no special attack. */

const PRIMARY_ATTACK_INTERVAL: f64 = 0.003;
const PROJ_COLOR: Color = Color::new(0.0, 1.6, 0.0, 1.0);
const PROJ_VERTEXES: [Point; 4] = [Point::new(-0.2, -0.2), Point::new(0.2, -0.2), Point::new(0.2, 0.2), Point::new(-0.2, 0.2)];

struct Weapon1Data {
    since_last_primary_attack: f64,
}

pub fn new_weapon1() -> Weapon {
    let ws_data = Box::new(Weapon1Data {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
    });
    Weapon::new(ws_data, Box::new(handle_tick_fn), Box::new(draw_hud), Box::new(draw_on_owner))
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<Weapon1Data>().unwrap();
    let mut response = WeaponHandleTickResponse::new();
    response.damage_color = DamageColor::Green;

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
        vertexes: Box::new(PROJ_VERTEXES),
    };
    let proj = Projectile2Builder::new(
        Projectile2BuilderReq {
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
        }
    ).build(ctx.nro_ctx);
    
    response.new_room_objs.push(Rc::new(RefCell::new(proj)));
    response
}

fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let weapon_draw_op = draw_op_rect((&PROJ_COLOR).into(), ctx.x, ctx.y, ctx.scale_height, ctx.scale_height);
    DrawWeaponHudResponse { weapon_draw_op, ammo_text: "ammo_text".to_owned() }
}

fn draw_on_owner(ctx: &DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse {
    let mut dop_vertexes = [Point::default(); 4];
    dop_vertexes.copy_from_slice(&PROJ_VERTEXES);
    translate_polygon(Vector::new(ctx.x, ctx.y), &mut dop_vertexes);
    DrawWeaponOnOwnerResponse {
        draw_op: ctx.draw_ctx.do_quad_fan(PROJ_COLOR, dop_vertexes),
        owner_color: Color::new(0.0, 1.0, 0.0, 1.0),
    }
}