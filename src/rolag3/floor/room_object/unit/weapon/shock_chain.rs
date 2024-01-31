use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::{rolag3::floor::{room_object::{damage::DamageColor, unit::standard_unit1::StandardUnit1, room_object_def::{NewRoomObjectContext, Act1Response, RoomObject}}, draw::Color}, gfx::{draw_op_util::draw_op_annulus, renderer::{DrawOpGroup, DrawOp}}, geometry::shape::Point, util::lerp::lerp_f64};

use super::{weapon_def::{Weapon, WeaponHandleTickContext, WeaponHandleTickResponse, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerResponse, DrawWeaponOnOwnerContext, BuyAmmoInfo, SwitchOutWeaponResponse, SwitchOutWeaponContext, WeaponExitRoomResponse, WeaponExitRoomContext}, shock_chain_link::new_shock_chain_link};

/* Shock Harpoon shoots 
*/

const NAME: &str = "Shock Chain";
const SHOP_DESCRIPTION: &str = "Launches a short-range chain that damages and slows enemies it touches";
const SHOP_COST: u64 = 20;
const STARTING_AMMO: f64 = 1e3;
const BUY_AMMO_INFO: BuyAmmoInfo = BuyAmmoInfo { ammo_amount: 50.0, starcash_cost: 2.0 };

const PRIMARY_ATTACK_INTERVAL: f64 = 0.9;
const ATTACK_DURATION: f64 = 0.7;
const NUM_LINKS: usize = 8;
const LINK_COLOR: Color = Color::new(0.0, 2.0, 0.1, 0.9);
const LINK_RADIUS: f32 = 0.5;
const DPS_PER_LINK: f64 = 5.0;

struct ShockChainData {
    since_last_primary_attack: f64,
    attack: Option<Attack>,
}

struct Attack {
    links: Vec<Weak<RefCell<StandardUnit1>>>,
    link_positions: Vec<(f64, f64)>,
    start_owner_age: f64,
    end_owner_age: f64,
    target: (f64, f64),
}

pub fn new_weapon_shock_chain() -> Weapon {
    let ws_data = Box::new(ShockChainData {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
        attack: None,
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
        Box::new(switch_out),
        Box::new(exit_room),
        Box::new(draw_hud), 
        Box::new(draw_on_owner),
    )
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<ShockChainData>().unwrap();
    let mut response = WeaponHandleTickResponse::new();
    response.damage_color = DamageColor::Green;

    ws_data.since_last_primary_attack += ctx.tick_len;

    if let Some(attack) = &mut ws_data.attack {
        if ctx.owner_age >= attack.end_owner_age {
            for link_weak in attack.links.iter_mut() {
                let link_rc = link_weak.upgrade().unwrap();
                let link = link_rc.borrow();
                response.room_objs_to_remove.push(link.get_metadata().get_ref());
            }
            // TODO: fix bug where player moves rooms. Shock chain units are in the previous room and therefore
            // can't be deleted
            ws_data.attack = None;
        } else {
            assert!(attack.links.len() > 1);
            let t_01 = (ctx.owner_age - attack.start_owner_age) / (attack.end_owner_age - attack.start_owner_age);
            let lerp_t = 1.0 - 4.0 * f64::powi(t_01 - 0.5, 2);
            for (i, link_weak) in attack.links.iter_mut().enumerate() {
                let link_rc = link_weak.upgrade().unwrap();
                let mut link = link_rc.borrow_mut();
                let mut dummy_response = Act1Response::new();
                let local_lerp_t = lerp_t * (i as f64 / ((NUM_LINKS - 1) as f64));
                let x = lerp_f64(ctx.owner_xform.dx, attack.target.0, local_lerp_t);
                let y = lerp_f64(ctx.owner_xform.dy, attack.target.1, local_lerp_t);
                attack.link_positions[i] = (x, y);
                let input = (x, y);
                let mut output = ();
                link.slave_act1_fn(ctx.act1_ctx, &mut dummy_response, &input, &mut output);
            }
            return response;
        }
    }

    if !ctx.primary_attack {
        return response;
    }
    if ws_data.since_last_primary_attack < PRIMARY_ATTACK_INTERVAL || *ctx.ammo < 1.0 {
        return response;
    }
    *ctx.ammo -= 1.0;
    ws_data.since_last_primary_attack = 0.0;

    let mut links = Vec::new();
    for _ in 0..NUM_LINKS {
        let nro_ctx = &mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
        let link = new_shock_chain_link(
            nro_ctx, 
            ctx.owner_team, 
            ctx.owner_xform.dx, 
            ctx.owner_xform.dy, 
            DPS_PER_LINK, 
            LINK_RADIUS,
        );
        let link = Rc::new(RefCell::new(link));
        links.push(Rc::downgrade(&link));
        response.new_room_objs.push(link);
    }

    let angle = f64::atan2(ctx.mouse_y - ctx.owner_xform.dy, ctx.mouse_x - ctx.owner_xform.dx);
    let max_mag = 1.7 * (LINK_RADIUS as f64) * (NUM_LINKS as f64);
    let target_x = ctx.owner_xform.dx + max_mag * f64::cos(angle);
    let target_y = ctx.owner_xform.dy + max_mag * f64::sin(angle);

    ws_data.attack = Some(Attack {
        links,
        link_positions: vec![(ctx.owner_xform.dx, ctx.owner_xform.dy); NUM_LINKS],
        start_owner_age: ctx.owner_age,
        end_owner_age: ctx.owner_age + ATTACK_DURATION,
        target: (target_x, target_y),
    });

    let mut rng = ctx.act1_ctx.get_rng().spawn_child();
    let sound_candidates = &ctx.act1_ctx.get_sound_db().taser_stun_gun_zap_electricity;
    let sound_data = rng.sample_slice_uniform(sound_candidates);
    response.newly_played_sounds.push(ctx.act1_ctx.new_play_sound_builder(sound_data).playback_speed(0.5).build());

    response
}

fn switch_out(ctx: &mut SwitchOutWeaponContext) -> SwitchOutWeaponResponse {
    let mut response = SwitchOutWeaponResponse::new();
    let ws_data = ctx.ws_data.downcast_mut::<ShockChainData>().unwrap();
    if let Some(attack) = ws_data.attack.take() {
        for link_weak in attack.links {
            let link_rc = link_weak.upgrade().unwrap();
            let link = link_rc.borrow();
            response.room_objs_to_remove.push(link.get_metadata().get_ref());
        }
    }
    response
}

fn exit_room(ctx: &mut WeaponExitRoomContext) -> WeaponExitRoomResponse {
    let ws_data = ctx.ws_data.downcast_mut::<ShockChainData>().unwrap();
    let mut response = WeaponExitRoomResponse::new();
    if let Some(attack) = ws_data.attack.take() {
        for link_weak in attack.links {
            let link_rc = link_weak.upgrade().unwrap();
            let link = link_rc.borrow();
            response.room_objs_to_remove.push(link.get_metadata().get_ref());
        }
    }
    response
}

fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let center = (ctx.x + ctx.scale_height / 2.0, ctx.y + ctx.scale_height / 2.0);
    let radius = ctx.scale_height / 2.0;
    let weapon_draw_op = draw_op_annulus((&LINK_COLOR).into(), center, 0.8 * radius, radius);
    DrawWeaponHudResponse { 
        weapon_draw_op,
        ammo_text: format!("{}", ctx.ammo),
     }
}

fn draw_on_owner(ctx: &DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse {
    let ws_data = ctx.ws_data.downcast_ref::<ShockChainData>().unwrap();
    let mut draw_ops = Vec::new();
    draw_ops.push(ctx.draw_ctx.do_annulus(LINK_COLOR, Point::new(ctx.x, ctx.y), 0.8 * LINK_RADIUS, LINK_RADIUS));
    if let Some(ref attack) = ws_data.attack {
        for (x, y) in attack.link_positions.iter() {
            let center = Point::new(*x as f32, *y as f32);
            draw_ops.push(ctx.draw_ctx.do_annulus(LINK_COLOR, center, 0.8 * LINK_RADIUS, LINK_RADIUS))
        }
    }
    DrawWeaponOnOwnerResponse {
        draw_op: DrawOp::Group(DrawOpGroup::new(draw_ops.into())),
        owner_color: Color::new(0.0, 0.5, 0.1, 1.0),
    }
}