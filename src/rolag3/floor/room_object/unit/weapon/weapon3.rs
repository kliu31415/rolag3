use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{projectile::projectile2::{Projectile2BuilderReq, Proj2Shape, Projectile2Builder}, damage::DamageColor}, draw::Color}, gfx::draw_op_util::draw_op_circle, geometry::shape::Point};

use super::weapon_def::{Weapon, WeaponHandleTickContext, WeaponHandleTickResponse, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerResponse, DrawWeaponOnOwnerContext};

/* Weapon3 shoots a wave of 3 red circles at intervals of 0.1s. It has no special attack.
*/

const PRIMARY_ATTACK_INTERVAL: f64 = 0.1;
const PROJ_COLOR: Color = Color::new(6.0, 0.1, 0.1, 1.0);
const PROJ_RADIUS: f32 = 0.4;

struct Weapon3Data {
    since_last_primary_attack: f64,
}


pub fn new_weapon3() -> Weapon {
    let ws_data = Box::new(Weapon3Data {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
    });
    Weapon::new(ws_data, Box::new(handle_tick_fn), Box::new(draw_hud), Box::new(draw_on_owner))
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<Weapon3Data>().unwrap();
    let mut response = WeaponHandleTickResponse::new();
    response.damage_color = DamageColor::Red;

    ws_data.since_last_primary_attack += ctx.tick_len;
    if !ctx.primary_attack {
        return response;
    }
    if ws_data.since_last_primary_attack < PRIMARY_ATTACK_INTERVAL {
        return response;
    }
    ws_data.since_last_primary_attack = 0.0;

    let proj_velocity = 40.0;
    for i in -1..2 {
        let proj_angle = f64::atan2(ctx.mouse_y - ctx.owner_xform.dy, ctx.mouse_x - ctx.owner_xform.dx);
        let angle = proj_angle + (i as f64) * std::f64::consts::FRAC_PI_6;
        let velocity_x = ctx.owner_velocity_x + proj_velocity * f64::cos(angle);
        let velocity_y = ctx.owner_velocity_y + proj_velocity * f64::sin(angle);
        let shape = Proj2Shape::Circle {x: 0.0, y: 0.0, r: PROJ_RADIUS};
        let proj = Projectile2Builder::new(
            Projectile2BuilderReq{
                team: ctx.owner_team,
                damage_color: DamageColor::Red,
                damage: 3.0,
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
    }
    response
}


fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let center = (ctx.x + ctx.scale_height / 2.0, ctx.y + ctx.scale_height / 2.0);
    let radius = ctx.scale_height / 2.0;
    let weapon_draw_op = draw_op_circle((&PROJ_COLOR).into(), center, radius);
    DrawWeaponHudResponse { 
        weapon_draw_op,
        ammo_text: "ammo_text".to_owned(),
     }
}

fn draw_on_owner(ctx: &DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse {
    DrawWeaponOnOwnerResponse {
        draw_op: ctx.draw_ctx.do_circle(PROJ_COLOR, Point::new(ctx.x, ctx.y), PROJ_RADIUS),
        owner_color: Color::new(1.0, 0.1, 0.1, 1.0),
    }
}