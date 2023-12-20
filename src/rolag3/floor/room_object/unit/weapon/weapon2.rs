use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{draw::Color, room_object::{room_object_def::RoomObject, projectile::projectile2::{Proj2Shape, NewProjectile2Args}, damage::DamageColor}}, geometry::shape::Point, gfx::draw_op_util::draw_op_rect};

use super::weapon_def::{Weapon, WeaponHandleTickResponse, WeaponHandleTickContext, DrawWeaponHudContext, DrawWeaponHudResponse};

/* Weapon2 shoots a wave of 3 blue squares at intervals of 0.3s.
   It has a special attack, which when used, causes it to shoot much more rapidly.
*/

const PRIMARY_ATTACK_INTERVAL: f64 = 0.3;
const SPECIAL_ATTACK_COOLDOWN: f64 = 1.5;
const SPECIAL_ATTACK_MANA_COST: f64 = 2.0;

const PROJ_COLOR: Color = Color::new(0.2, 0.2, 15.0, 1.0);

struct Weapon2Data {
    since_last_primary_attack: f64,
    since_last_special_attack: f64,
}

pub fn new_weapon2() -> Weapon {
    let ws_data = Box::new(Weapon2Data {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
        since_last_special_attack: SPECIAL_ATTACK_COOLDOWN,
    });
    Weapon::new(ws_data, Box::new(handle_tick_fn), Box::new(draw_hud))
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<Weapon2Data>().unwrap();
    let mut response = WeaponHandleTickResponse::new();

    ws_data.since_last_primary_attack += ctx.tick_len;
    ws_data.since_last_special_attack += ctx.tick_len;

    if ctx.special_attack && ws_data.since_last_special_attack >= SPECIAL_ATTACK_COOLDOWN && ctx.owner_mana >= SPECIAL_ATTACK_MANA_COST {
        response.mana_delta -= SPECIAL_ATTACK_MANA_COST;
        ws_data.since_last_special_attack = 0.0;
        let maxi = 128;
        for i in 0..maxi {
            let angle_adjust = (i as f64) / (maxi as f64) * 2.0 * std::f64::consts::PI;
            response.new_room_objs.push(spawn_projectile(ctx, angle_adjust));
        }
        return response;
    }

    if !ctx.primary_attack {
        return response;
    }
    if ws_data.since_last_primary_attack < PRIMARY_ATTACK_INTERVAL {
        return response;
    }

    ws_data.since_last_primary_attack = 0.0;

    for i in -1..2 {
        let angle_adjust = (i as f64) * std::f64::consts::FRAC_PI_6;
        response.new_room_objs.push(spawn_projectile(ctx, angle_adjust));
    }

    response
}

fn spawn_projectile(ctx: &mut WeaponHandleTickContext, angle_adjust: f64) -> Rc<RefCell<dyn RoomObject>> {
    let proj_velocity = 60.0;
    let proj_angle = f64::atan2(ctx.mouse_y - ctx.owner_xform.dy, ctx.mouse_x - ctx.owner_xform.dx);
    let proj_angle = proj_angle + angle_adjust;
    let velocity_x = ctx.owner_velocity_x + proj_velocity * f64::cos(proj_angle);
    let velocity_y = ctx.owner_velocity_y + proj_velocity * f64::sin(proj_angle);
    let shape = Proj2Shape::TriFan {
        center: Point::new(0.0, 0.0), 
        vertexes: vec![Point::new(-0.2, -0.2), Point::new(0.2, -0.2), Point::new(0.2, 0.2), Point::new(-0.2, 0.2)].into_boxed_slice() 
    };
    let proj = NewProjectile2Args{
        team: ctx.owner_team,
        damage_color: DamageColor::Blue,
        damage: 7.0,
        owner: ctx.owner.clone(),
        lifespan: 2.0,
        velocity_x,
        velocity_y,
        xform: ctx.owner_xform,
        shape,
        color: PROJ_COLOR,
    }.new(ctx.nro_ctx);
    Rc::new(RefCell::new(proj))
}

fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let weapon_draw_op = draw_op_rect((&PROJ_COLOR).into(), ctx.x, ctx.y, ctx.scale_height, ctx.scale_height);
    DrawWeaponHudResponse { 
        weapon_draw_op,
        ammo_text: "∞".to_owned(),
    }
}