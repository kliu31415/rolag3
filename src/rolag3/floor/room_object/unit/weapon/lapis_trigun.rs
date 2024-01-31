use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{draw::Color, room_object::{room_object_def::{RoomObject, NewRoomObjectContext}, projectile::projectile2::{Proj2Shape, Projectile2BuilderReq, Projectile2Builder}, damage::DamageColor}}, geometry::{shape::{Point, Vector}, util::translate_polygon}, gfx::draw_op_util::draw_op_rect};

use super::weapon_def::{Weapon, WeaponHandleTickResponse, WeaponHandleTickContext, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerContext, DrawWeaponOnOwnerResponse, BuyAmmoInfo, SwitchOutWeaponResponse};

/* Lapis Trigun shoots a wave of 3 blue squares at intervals of 0.3s.
   It has a special attack, which when used, causes it to shoot a radial wave of 128 projectiles.
*/

const SP_ATK_PROJ_COUNT: usize = 128;
const NAME: &str = "Lapis Trigun";
const SHOP_DESCRIPTION: &str = "Fires waves of three projectiles. 
Special attack (Gem Storm): ejects a radial wave of 128 projectiles";
const SHOP_COST: u64 = 10;
const STARTING_AMMO: f64 = 1e2;
const BUY_AMMO_INFO: BuyAmmoInfo = BuyAmmoInfo { ammo_amount: 100.0, starcash_cost: 2.0 };

const PRIMARY_ATTACK_INTERVAL: f64 = 0.3;
const SPECIAL_ATTACK_COOLDOWN: f64 = 1.5;
const SPECIAL_ATTACK_MANA_COST: f64 = 2.0;

const PROJ_VERTEXES: [Point; 4] = [Point::new(-0.2, -0.2), Point::new(0.2, -0.2), Point::new(0.2, 0.2), Point::new(-0.2, 0.2)];
const PROJ_COLOR: Color = Color::new(0.2, 0.2, 15.0, 1.0);

struct Weapon2Data {
    since_last_primary_attack: f64,
    since_last_special_attack: f64,
}

pub fn new_weapon_lapis_trigun() -> Weapon {
    let ws_data = Box::new(Weapon2Data {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
        since_last_special_attack: SPECIAL_ATTACK_COOLDOWN,
    });
    Weapon::new(
        DamageColor::Blue,
        NAME,
        SHOP_DESCRIPTION,
        SHOP_COST,
        STARTING_AMMO,
        Some(BUY_AMMO_INFO),
        ws_data, 
        Box::new(handle_tick_fn), 
        Box::new(|_| SwitchOutWeaponResponse::new()),
        Box::new(draw_hud), 
        Box::new(draw_on_owner),
    )
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<Weapon2Data>().unwrap();
    let mut response = WeaponHandleTickResponse::new();
    response.damage_color = DamageColor::Blue;

    ws_data.since_last_primary_attack += ctx.tick_len;
    ws_data.since_last_special_attack += ctx.tick_len;

    if ctx.special_attack && ws_data.since_last_special_attack >= SPECIAL_ATTACK_COOLDOWN && ctx.owner_mana >= SPECIAL_ATTACK_MANA_COST {
        response.mana_delta -= SPECIAL_ATTACK_MANA_COST;
        ws_data.since_last_special_attack = 0.0;
        for i in 0..SP_ATK_PROJ_COUNT {
            let angle_adjust = (i as f64) / (SP_ATK_PROJ_COUNT as f64) * 2.0 * std::f64::consts::PI;
            response.new_room_objs.push(spawn_projectile(ctx, angle_adjust));
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

    for i in -1..2 {
        let angle_adjust = (i as f64) * std::f64::consts::FRAC_PI_6;
        response.new_room_objs.push(spawn_projectile(ctx, angle_adjust));
    }
    let mut rng = ctx.act1_ctx.get_rng().spawn_child();
    let sound_candidates = &ctx.act1_ctx.get_sound_db().gun_pistol_shot;
    let sound_data = rng.sample_slice_uniform(sound_candidates);
    response.newly_played_sounds.push(ctx.act1_ctx.new_play_sound_builder(sound_data).build());

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
        vertexes: Box::new(PROJ_VERTEXES),
    };
    let nro_ctx = &mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
    let proj = Projectile2Builder::new(
        Projectile2BuilderReq {
            team: ctx.owner_team,
            damage_color: DamageColor::Blue,
            damage: 7.0,
            owner: ctx.owner.clone(),
            velocity_x,
            velocity_y,
            xform: ctx.owner_xform,
            shape,
            color: PROJ_COLOR,
        }
    ).build(nro_ctx);
    Rc::new(RefCell::new(proj))
}

fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let weapon_draw_op = draw_op_rect((&PROJ_COLOR).into(), ctx.x, ctx.y, ctx.scale_height, ctx.scale_height);
    DrawWeaponHudResponse { 
        weapon_draw_op,
        ammo_text: format!("{}", ctx.ammo),
    }
}

fn draw_on_owner(ctx: &DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse {
    let mut dop_vertexes = [Point::default(); 4];
    dop_vertexes.copy_from_slice(&PROJ_VERTEXES);
    translate_polygon(Vector::new(ctx.x, ctx.y), &mut dop_vertexes);
    DrawWeaponOnOwnerResponse {
        draw_op: ctx.draw_ctx.do_quad_fan(PROJ_COLOR, dop_vertexes),
        owner_color: Color::new(0.1, 0.1, 3.0, 1.0),
    }
}