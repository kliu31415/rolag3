use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{projectile::projectile2::{Proj2Shape, Projectile2BuilderReq, Projectile2Builder}, damage::DamageColor, room_object_def::new_play_sound_builder}, draw::Color, rofiz::rofiz_object::Transformation}, geometry::{shape::{Point, Vector}, util::translate_polygon}, gfx::draw_op_util::draw_op_rect, util::token_bucket::TokenBucket};

use super::weapon_def::{WeaponHandleTickContext, Weapon, WeaponHandleTickResponse, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerContext, DrawWeaponOnOwnerResponse, BuyAmmoInfo};

/* Green Laser rapidly shoots green squares, like a laser. It has no special attack. */

const NAME: &str = "Green Laser";
const SHOP_DESCRIPTION: &str = "Shoots a rapid, continuous laser beam";
const SHOP_COST: u64 = 50;
const STARTING_AMMO: f64 = 1e4;
const BUY_AMMO_INFO: BuyAmmoInfo = BuyAmmoInfo { ammo_amount: 1000.0, starcash_cost: 8.0 };

const PRIMARY_ATTACK_INTERVAL: f64 = 0.0025;
const PROJ_COLOR: Color = Color::new(0.0, 1.6, 0.0, 1.0);
const PROJ_VERTEXES: [Point; 4] = [Point::new(-0.2, -0.2), Point::new(0.2, -0.2), Point::new(0.2, 0.2), Point::new(-0.2, 0.2)];

struct Weapon1Data {
    since_last_primary_attack: f64,
    sound_token_bucken: TokenBucket,
}

pub fn new_weapon_green_laser() -> Weapon {
    let ws_data = Box::new(Weapon1Data {
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
        Box::new(draw_hud), 
        Box::new(draw_on_owner),
    )
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

    while ws_data.since_last_primary_attack >= PRIMARY_ATTACK_INTERVAL && *ctx.ammo >= 1.0 {
        ws_data.since_last_primary_attack -= PRIMARY_ATTACK_INTERVAL;
        *ctx.ammo -= 1.0;

        let proj_velocity = 120.0;
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
            0.0,
        );
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
        ).lifespan(2.0)
            .build(ctx.nro_ctx);
        response.new_room_objs.push(Rc::new(RefCell::new(proj)));
    }

    if ws_data.sound_token_bucken.try_take_fok(ctx.owner_age, 1.0) {
        let sound_data = ctx.nro_ctx.get_rng().sample_slice_uniform(&ctx.sound_db.sci_fi_weapon_laser_small);
        response.newly_played_sounds.push(new_play_sound_builder(ctx.sound_id_counter, sound_data).build());
    }

    response
}

fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let weapon_draw_op = draw_op_rect((&PROJ_COLOR).into(), ctx.x, ctx.y, ctx.scale_height, ctx.scale_height);
    DrawWeaponHudResponse { weapon_draw_op, ammo_text: format!("{}", ctx.ammo) }
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