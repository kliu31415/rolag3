use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{projectile::{projectile2::{Projectile2BuilderReq, Proj2Shape, Projectile2Builder, Explosion1OnDeathFnArgs}, explosion1::{Explosion1, new_explosion1}}, damage::DamageColor}, draw::Color}, gfx::draw_op_util::draw_op_circle, geometry::shape::{Point, Vector}};

use super::weapon_def::{Weapon, WeaponHandleTickContext, WeaponHandleTickResponse, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerResponse, DrawWeaponOnOwnerContext, BuyAmmoInfo};

/* Ruby Rockets shoots a red rocket that explodes
*/

const NAME: &str = "Ruby Rockets";
const SHOP_DESCRIPTION: &str = "Fires a rocket";
const STARTING_AMMO: f64 = 1e2;
const BUY_AMMO_INFO: BuyAmmoInfo = BuyAmmoInfo { ammo_amount: 50.0, starcash_cost: 5.0 };

const PRIMARY_ATTACK_INTERVAL: f64 = 0.7;
const PROJ_COLOR: Color = Color::new(6.0, 0.05, 0.05, 1.0);
const PROJ_VERTEXES: [Point; 3] = [Point::new(0.5, 0.0), Point::new(-0.3, -0.3), Point::new(-0.3, 0.3)];

struct RubyRockets {
    since_last_primary_attack: f64,
}

pub fn new_weapon_ruby_rockets() -> Weapon {
    let ws_data = Box::new(RubyRockets {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
    });
    Weapon::new(
        NAME,
        SHOP_DESCRIPTION,
        STARTING_AMMO,
        Some(BUY_AMMO_INFO),
        ws_data, 
        Box::new(handle_tick_fn), 
        Box::new(draw_hud), 
        Box::new(draw_on_owner),
    )
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<RubyRockets>().unwrap();
    let mut response = WeaponHandleTickResponse::new();
    response.damage_color = DamageColor::Red;

    ws_data.since_last_primary_attack += ctx.tick_len;
    if !ctx.primary_attack {
        return response;
    }
    if ws_data.since_last_primary_attack < PRIMARY_ATTACK_INTERVAL || *ctx.ammo < 1.0 {
        return response;
    }
    *ctx.ammo -= 1.0;
    ws_data.since_last_primary_attack = 0.0;

    let proj_velocity = 70.0;
    for i in 0..=0 {
        let proj_angle = f64::atan2(ctx.mouse_y - ctx.owner_xform.dy, ctx.mouse_x - ctx.owner_xform.dx);
        let angle = proj_angle + (i as f64) * std::f64::consts::FRAC_PI_6;
        let velocity_x = ctx.owner_velocity_x + proj_velocity * f64::cos(angle);
        let velocity_y = ctx.owner_velocity_y + proj_velocity * f64::sin(angle);
        let shape = Proj2Shape::TriFan { center: Point::new(0.0, 0.0), vertexes: Box::new(PROJ_VERTEXES) };
        let new_explosion1_fn = |args: Explosion1OnDeathFnArgs| -> Explosion1 {
            let lifespan = 0.5;
            let radius_fn = move |age: f64| {
                5.0 * f64::cbrt(age / lifespan)
            };
            new_explosion1(args.nro_ctx, 
                args.team, 
                args.owner, 
                DamageColor::Red, 
                args.x, 
                args.y, 
                10.0, 
                lifespan, 
                Color::new(10.0, 0.1, 0.1, 0.6), 
                Color::new(10.0, 1.0, 1.0, 0.01), 
                Box::new(radius_fn),
            )
        };
        let proj = Projectile2Builder::new(
            Projectile2BuilderReq{
                team: ctx.owner_team,
                damage_color: DamageColor::Red,
                damage: 3.0,
                owner: ctx.owner.clone(),
                velocity_x,
                velocity_y,
                xform: ctx.owner_xform,
                shape,
                color: PROJ_COLOR,
            }
        ).explosion1_on_death_fn(Box::new(new_explosion1_fn))
            .build(ctx.nro_ctx);
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
        ammo_text: format!("{}", ctx.ammo),
     }
}

fn draw_on_owner(ctx: &DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse {
    let vertexes = PROJ_VERTEXES.map(|p| p + Vector::new(ctx.x, ctx.y));
    DrawWeaponOnOwnerResponse {
        draw_op: ctx.draw_ctx.do_tri(PROJ_COLOR, vertexes),
        owner_color: Color::new(1.0, 0.1, 0.1, 1.0),
    }
}