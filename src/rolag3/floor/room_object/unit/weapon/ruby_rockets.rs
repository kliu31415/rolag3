use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{projectile::{projectile2::{Projectile2BuilderReq, Proj2Shape, Projectile2Builder, Explosion1OnDeathFnArgs}, explosion1::{Explosion1, new_explosion1}}, damage::DamageColor, room_object_def::RoomObject}, draw::Color, rofiz::rofiz_object::Transformation}, gfx::renderer::{DrawOp, DrawOpTri, ColoredTriVertex, ColorRGBA32f, ViewSpaceCoordinate}, geometry::shape::{Point, Vector}};

use super::weapon_def::{Weapon, WeaponHandleTickContext, WeaponHandleTickResponse, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerResponse, DrawWeaponOnOwnerContext, BuyAmmoInfo};

/* Ruby Rockets shoots a red rocket that explodes
*/

const NAME: &str = "Ruby Rockets";
const SHOP_DESCRIPTION: &str = "Fires exploding rockets at a moderate pace
Special attack (Rocket Rain): fires a radial wave of 64 rockets";
const SHOP_COST: u64 = 20;
const STARTING_AMMO: f64 = 1e2;
const BUY_AMMO_INFO: BuyAmmoInfo = BuyAmmoInfo { ammo_amount: 50.0, starcash_cost: 5.0 };

const PRIMARY_ATTACK_INTERVAL: f64 = 0.7;
const SPECIAL_ATTACK_COOLDOWN: f64 = 0.5;
const SPECIAL_ATTACK_MANA_COST: f64 = 5.0;
const PROJ_COLOR: Color = Color::new(6.0, 0.05, 0.05, 1.0);
const PROJ_VERTEXES: [Point; 3] = [Point::new(0.5, 0.0), Point::new(-0.3, -0.3), Point::new(-0.3, 0.3)];
const PROJ_VELOCITY: f64 = 50.0;

struct RubyRockets {
    since_last_primary_attack: f64,
    since_last_special_attack: f64,
}

pub fn new_weapon_ruby_rockets() -> Weapon {
    let ws_data = Box::new(RubyRockets {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
        since_last_special_attack: SPECIAL_ATTACK_COOLDOWN,
    });
    Weapon::new(
        DamageColor::Red,
        NAME,
        SHOP_DESCRIPTION,
        SHOP_COST,
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
    ws_data.since_last_special_attack += ctx.tick_len;

    if ctx.special_attack && ws_data.since_last_special_attack >= SPECIAL_ATTACK_COOLDOWN && ctx.owner_mana >= SPECIAL_ATTACK_MANA_COST {
        response.mana_delta -= SPECIAL_ATTACK_MANA_COST;
        ws_data.since_last_special_attack = 0.0;
        let num_proj = 64;
        for i in 0..num_proj {
            let angle = (i as f64) / (num_proj as f64) * 2.0 * std::f64::consts::PI;
            response.new_room_objs.push(spawn_projectile(ctx, angle, 3.0));
        }
        return response;
    }
    if !ctx.primary_attack {
        return response;
    }
    if ws_data.since_last_primary_attack < PRIMARY_ATTACK_INTERVAL || *ctx.ammo < 1.0 {
        return response;
    }
    *ctx.ammo -= 1.0;
    ws_data.since_last_primary_attack = 0.0;

    let angle = f64::atan2(ctx.mouse_y - ctx.owner_xform.dy, ctx.mouse_x - ctx.owner_xform.dx);
    let proj = spawn_projectile(ctx, angle, 4.0);
    response.new_room_objs.push(proj);
    response
}

fn spawn_projectile(ctx: &mut WeaponHandleTickContext, angle: f64, explosion_radius: f64) -> Rc<RefCell<dyn RoomObject>> {
    let velocity_x = ctx.owner_velocity_x + PROJ_VELOCITY * f64::cos(angle);
    let velocity_y = ctx.owner_velocity_y + PROJ_VELOCITY * f64::sin(angle);
    let shape = Proj2Shape::TriFan { center: Point::new(0.0, 0.0), vertexes: Box::new(PROJ_VERTEXES) };
    let new_explosion1_fn = move |args: Explosion1OnDeathFnArgs| -> Explosion1 {
        let stop_expand_at = 0.5;
        let lifespan = 0.65;
        assert!(stop_expand_at < lifespan);
        let radius_fn = move |age: f64| {
            explosion_radius * f64::cbrt(f64::min(1.0, age / stop_expand_at))
        };
        let outer_color_fn = move |age| {
            let mut color = Color::new(10.0, 0.1, 0.1, 0.6);
            if age > stop_expand_at {
                color.a *= ((lifespan - age) / (lifespan - stop_expand_at)) as f32;
            }
            color
        };
        let inner_color_fn = move |age| {
            let mut color = Color::new(10.0, 1.0, 1.0, 0.01);
            if age > stop_expand_at {
                color.a *= ((lifespan - age) / (lifespan - stop_expand_at)) as f32;
            }
            color
        };
        new_explosion1(args.nro_ctx, 
            args.team, 
            args.owner, 
            DamageColor::Red, 
            args.x, 
            args.y, 
            10.0, 
            lifespan, 
            Box::new(outer_color_fn), 
            Box::new(inner_color_fn), 
            Box::new(radius_fn),
        )
    };
    let proj_xform: Transformation = Transformation::new(ctx.owner_xform.dx, ctx.owner_xform.dy, f64::atan2(velocity_y, velocity_x));
    let proj = Projectile2Builder::new(
        Projectile2BuilderReq{
            team: ctx.owner_team,
            damage_color: DamageColor::Red,
            damage: 3.0,
            owner: ctx.owner.clone(),
            velocity_x,
            velocity_y,
            xform: proj_xform,
            shape,
            color: PROJ_COLOR,
        }
    ).explosion1_on_death_fn(Box::new(new_explosion1_fn))
        .build(ctx.nro_ctx);
    Rc::new(RefCell::new(proj))
}


fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let center = Vector::new(ctx.x + ctx.scale_height / 2.0, ctx.y + ctx.scale_height / 2.0);
    let vertexes = PROJ_VERTEXES
        .map(|p| Point::new(p.x * ctx.scale_height, p.y * ctx.scale_height))
        .map(|p| p + center)
        .map(|p| 
            ColoredTriVertex { 
                color: ColorRGBA32f::new(PROJ_COLOR.r, PROJ_COLOR.g, PROJ_COLOR.b, PROJ_COLOR.a), 
                vertex: ViewSpaceCoordinate::new(p.x, p.y),
            }
        );
    let weapon_draw_op = DrawOp::Tri(DrawOpTri { vertexes });
    DrawWeaponHudResponse { 
        weapon_draw_op,
        ammo_text: format!("{}", ctx.ammo),
     }
}

fn draw_on_owner(ctx: &DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse {
    let ws_data = ctx.ws_data.downcast_ref::<RubyRockets>().unwrap();
    let angle = f64::atan2(ctx.mouse_y_game_coords - ctx.owner_xform.dy, ctx.mouse_x_game_coords - ctx.owner_xform.dx) as f32;
    let vertexes = PROJ_VERTEXES.map(|p| p.rotated(angle).translated(Vector::new(ctx.x, ctx.y)));

    let lerp_t = f64::min(ws_data.since_last_primary_attack / PRIMARY_ATTACK_INTERVAL, 1.0) as f32;
    let inner_color = Color::lerp(Color::new(0.1, 0.0, 0.0, 1.0), PROJ_COLOR, lerp_t);

    DrawWeaponOnOwnerResponse {
        draw_op: ctx.draw_ctx.do_tri(inner_color, vertexes),
        owner_color: Color::new(0.3, 0.01, 0.01, 1.0),
    }
}