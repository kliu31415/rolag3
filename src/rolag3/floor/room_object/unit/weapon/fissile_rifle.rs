use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{projectile::projectile2::{Proj2Shape, Projectile2BuilderReq, Projectile2Builder}, damage::DamageColor, room_object_def::NewRoomObjectContext}, draw::Color, rofiz::rofiz_object::Transformation}, geometry::shape::{Point, Vector}, gfx::renderer::{ColoredTriVertex, ColorRGBA32f, ViewSpaceCoordinate, DrawOp, DrawOpQuadFan}, util::token_bucket::TokenBucket};

use super::weapon_def::{WeaponHandleTickContext, Weapon, WeaponHandleTickResponse, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerContext, DrawWeaponOnOwnerResponse, BuyAmmoInfo, SwitchOutWeaponResponse, WeaponExitRoomResponse};

/* Fissile Rifle shoots homing porjectiles that cause an explosion when enough hit the same target in a short timespan */

const NAME: &str = "Fissile Rifle";
const SHOP_DESCRIPTION: &str = "Shoots homing rays of fissile material";
const SHOP_COST: u64 = 20;
const STARTING_AMMO: f64 = 1e4;
const BUY_AMMO_INFO: BuyAmmoInfo = BuyAmmoInfo { ammo_amount: 100.0, starcash_cost: 2.0 };

const PRIMARY_ATTACK_INTERVAL: f64 = 0.3;
const PROJ_COLOR: Color = Color::new(0.0, 1.6, 0.2, 1.0);
const PROJ_VERTEXES: [Point; 4] = [
    Point::new(0.4, 0.0), 
    Point::new(-0.4, -0.4), 
    Point::new(0.0, 0.0), 
    Point::new(-0.4, 0.4),
];

struct FissileRifle {
    since_last_primary_attack: f64,
    sound_token_bucken: TokenBucket,
}

pub fn new_weapon_fissile_rifle() -> Weapon {
    let ws_data = Box::new(FissileRifle {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
        sound_token_bucken: TokenBucket::new(1.0, 5.0),
    });
    Weapon::new(
        DamageColor::Green,
        NAME,
        SHOP_DESCRIPTION,
        SHOP_COST,
        STARTING_AMMO,
        Some(BUY_AMMO_INFO),
        ws_data, 
        Box::new(handle_tick_fn), 
        Box::new(|_| SwitchOutWeaponResponse::new()),
        Box::new(|_| WeaponExitRoomResponse::new()),
        Box::new(draw_hud), 
        Box::new(draw_on_owner),
    )
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<FissileRifle>().unwrap();
    let mut response = WeaponHandleTickResponse::new();
    response.damage_color = DamageColor::Green;

    ws_data.since_last_primary_attack += ctx.tick_len;
    if !ctx.primary_attack {
        return response;
    }
    if ws_data.since_last_primary_attack < PRIMARY_ATTACK_INTERVAL || *ctx.ammo < 1.0 {
        return response;
    }
    *ctx.ammo -= 1.0;
    ws_data.since_last_primary_attack = 0.0;

    let proj_velocity = 20.0;
    let proj_angle = f64::atan2(ctx.mouse_y - ctx.owner_xform.dy, ctx.mouse_x - ctx.owner_xform.dx);
    let velocity_x = ctx.owner_velocity_x + proj_velocity * f64::cos(proj_angle);
    let velocity_y = ctx.owner_velocity_y + proj_velocity * f64::sin(proj_angle);
    let shape = Proj2Shape::TriFan {
        center: Point::new(0.0, 0.0), 
        vertexes: Box::new(PROJ_VERTEXES),
    };
    let proj_xform = Transformation::new(
        ctx.owner_xform.dx + ws_data.since_last_primary_attack * velocity_x,
        ctx.owner_xform.dy + ws_data.since_last_primary_attack * velocity_y,
        proj_angle,
    );
    let homing_rotate_f = |_, proj_xform: Transformation, (enemy_x, enemy_y)| -> f64 {
        let dx = enemy_x - proj_xform.dx;
        let dy = enemy_y - proj_xform.dy;
        let dxy_norm = f64::hypot(dx, dy);
        // don't home if the projectile is too close or far to effectively home
        if !(0.1 .. 6.0).contains(&dxy_norm) {
            return 0.0;
        }

        let dx_normed = dx / dxy_norm;
        let dy_normed = dy / dx_normed;
        let dot = dx_normed * f64::cos(proj_xform.dtheta) + dy_normed * f64::sin(proj_xform.dtheta);
        if dot > 0.8 {
            return 4.0;
        }
        0.0
    };
    let nro_ctx = &mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
    let proj = Projectile2Builder::new(
        Projectile2BuilderReq {
            team: ctx.owner_team,
            damage_color: DamageColor::Green,
            damage: 2.0,
            owner: ctx.owner.clone(),
            velocity_x,
            velocity_y,
            xform: proj_xform,
            shape,
            color: PROJ_COLOR,
        }
    ).homing_rotate_to_enemies_speed_fn(Box::new(homing_rotate_f))
        .build(nro_ctx);
    response.new_room_objs.push(Rc::new(RefCell::new(proj)));

    if ws_data.sound_token_bucken.try_take_fok(ctx.owner_age, 1.0) {
        let mut rng = ctx.act1_ctx.get_rng().spawn_child();
        let sound_candidates = &ctx.act1_ctx.get_sound_db().sci_fi_weapon_laser_small_fun;
        let sound_data = rng.sample_slice_uniform(sound_candidates);
        response.newly_played_sounds.push(ctx.act1_ctx.new_play_sound_builder(sound_data).build());
    }

    response
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
    let weapon_draw_op = DrawOp::QuadFan(DrawOpQuadFan { vertexes });
    DrawWeaponHudResponse { 
        weapon_draw_op,
        ammo_text: format!("{}", ctx.ammo),
     }
}

fn draw_on_owner(ctx: &DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse {
    let ws_data = ctx.ws_data.downcast_ref::<FissileRifle>().unwrap();
    let angle = f64::atan2(ctx.mouse_y_game_coords - ctx.owner_xform.dy, ctx.mouse_x_game_coords - ctx.owner_xform.dx) as f32;
    let vertexes = PROJ_VERTEXES.map(|p| p.rotated(angle).translated(Vector::new(ctx.x, ctx.y)));

    let lerp_t = f64::min(ws_data.since_last_primary_attack / PRIMARY_ATTACK_INTERVAL, 1.0) as f32;
    let inner_color = Color::lerp(Color::new(0.1, 0.0, 0.0, 1.0), PROJ_COLOR, lerp_t);

    DrawWeaponOnOwnerResponse {
        draw_op: ctx.draw_ctx.do_quad_fan(inner_color, vertexes),
        owner_color: Color::new(0.0, 0.5, 0.0, 1.0),
    }
}