use std::{rc::{Rc, Weak}, cell::RefCell, collections::VecDeque};

use crate::{rolag3::floor::{draw::Color, room_object::{room_object_def::{RoomObject, NewRoomObjectContext, Act1Response}, damage::DamageColor, unit::standard_unit1::StandardUnit1}}, geometry::shape::{Point, Vector}, gfx::{draw_op_util::draw_op_annulus, renderer::{DrawOp, DrawOpGroup}}};

use super::{weapon_def::{Weapon, WeaponHandleTickResponse, WeaponHandleTickContext, DrawWeaponHudContext, DrawWeaponHudResponse, DrawWeaponOnOwnerContext, DrawWeaponOnOwnerResponse, BuyAmmoInfo, SwitchOutWeaponResponse, WeaponExitRoomResponse, SwitchOutWeaponContext, WeaponExitRoomContext}, sword_slash::new_sword_slash};

/* Azure Katana swipes in a 120 degree angle, deflecting projectiles
*/

const NAME: &str = "Azure Katana";
const SHOP_DESCRIPTION: &str = "Short-range melee weapon";
const SHOP_COST: u64 = 10;
const STARTING_AMMO: f64 = 1e2;
const BUY_AMMO_INFO: BuyAmmoInfo = BuyAmmoInfo { ammo_amount: 100.0, starcash_cost: 2.0 };

const SLASH_PART_RADIANS_EACH: f32 = 0.1;
const SLASH_PART_DPS: f64 = 3.0;
const SLASH_PART_RADIUS: f32 = 4.5;
const SLASH_DELTA: f64 = 2.0 * std::f64::consts::FRAC_PI_3;

const PRIMARY_ATTACK_INTERVAL: f64 = 0.6;

const SLASH_COLOR: Color = Color::new(0.0, 0.2, 20.0, 1.0);

struct AzureKatanaData {
    since_last_primary_attack: f64,
    attack: Option<Attack>,
}

struct Attack {
    start_owner_age: f64,
    start_angle: f64,
    total_angular_delta: f64,
    angular_speed: f64,
    next_slash_part_idx: usize,
    slash_parts: VecDeque<SlashPart>,
}

struct SlashPart {
    start_age: f64,
    end_age: f64,
    vertexes: [Point; 3],
    unit: Weak<RefCell<StandardUnit1>>,
}

pub fn new_weapon_azure_katana() -> Weapon {
    let ws_data = Box::new(AzureKatanaData {
        since_last_primary_attack: PRIMARY_ATTACK_INTERVAL,
        attack: None,
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
        Box::new(switch_out),
        Box::new(exit_room),
        Box::new(draw_hud), 
        Box::new(draw_on_owner),
    )
}

fn handle_tick_fn(ctx: &mut WeaponHandleTickContext) -> WeaponHandleTickResponse {
    let ws_data = ctx.ws_data.downcast_mut::<AzureKatanaData>().unwrap();
    let mut response = WeaponHandleTickResponse::new();
    response.damage_color = DamageColor::Blue;

    ws_data.since_last_primary_attack += ctx.tick_len;

    if let Some(attack) = &mut ws_data.attack {
        let cur_slash_delta = f64::min(attack.angular_speed * (ctx.owner_age - attack.start_owner_age), attack.total_angular_delta) as f32;
        if cur_slash_delta >= (attack.total_angular_delta as f32) && attack.slash_parts.is_empty() {
            ws_data.attack = None;
            return response;
        }

        loop {
            let next_part_at = (0.5 + attack.next_slash_part_idx as f32) * SLASH_PART_RADIANS_EACH;
            if next_part_at > cur_slash_delta {
                break;
            }
            let angle1 = (attack.start_angle as f32) + SLASH_PART_RADIANS_EACH * (attack.next_slash_part_idx as f32);
            let angle2 = angle1 + SLASH_PART_RADIANS_EACH;
            let vertexes = [
                Point::new(0.0, 0.0),
                Point::new(SLASH_PART_RADIUS * f32::cos(angle1), SLASH_PART_RADIUS * f32::sin(angle1)),
                Point::new(SLASH_PART_RADIUS * f32::cos(angle2), SLASH_PART_RADIUS * f32::sin(angle2))
            ];
            let slash_part = new_sword_slash(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx), 
                ctx.owner_team, 
                SLASH_PART_DPS, 
                ctx.owner_xform.dx, 
                ctx.owner_xform.dy, 
                Box::new(vertexes),
            );
            let slash_part = Rc::new(RefCell::new(slash_part));
            attack.slash_parts.push_back(SlashPart {
                unit: Rc::downgrade(&slash_part),
                start_age: ctx.owner_age,
                end_age: ctx.owner_age + 0.2,
                vertexes,
            });
            response.new_room_objs.push(slash_part);

            attack.next_slash_part_idx += 1;
        }

        while let Some(front) = attack.slash_parts.front() {
            if ctx.owner_age > front.end_age {
                let front_part_rc = front.unit.upgrade().unwrap();
                let front_part = front_part_rc.borrow();
                response.room_objs_to_remove.push(front_part.get_metadata().get_ref());
                attack.slash_parts.pop_front();
            } else {
                break;
            }
        }

        for part in attack.slash_parts.iter_mut() {
            let unit_rc = part.unit.upgrade().unwrap();
            let mut unit = unit_rc.borrow_mut();
            let mut dummy_response = Act1Response::new();
            let input = (ctx.owner_xform.dx, ctx.owner_xform.dy);
            let mut output = ();
            unit.slave_act1_fn(ctx.act1_ctx, &mut dummy_response, &input, &mut output);
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

    ws_data.attack = Some(Attack {
        start_owner_age: ctx.owner_age,
        start_angle: angle - 0.5 * SLASH_DELTA,
        total_angular_delta: SLASH_DELTA,
        angular_speed: 20.0,
        next_slash_part_idx: 0,
        slash_parts: VecDeque::new(),
    });

    let mut rng = ctx.act1_ctx.get_rng().spawn_child();
    let sound_candidates = &ctx.act1_ctx.get_sound_db().sci_fi_shield_device_small;
    let sound_data = rng.sample_slice_uniform(sound_candidates);
    response.newly_played_sounds.push(ctx.act1_ctx.new_play_sound_builder(sound_data).playback_speed(2.0).build());

    response
}

fn switch_out(ctx: &mut SwitchOutWeaponContext) -> SwitchOutWeaponResponse {
    let mut response = SwitchOutWeaponResponse::new();
    let ws_data = ctx.ws_data.downcast_mut::<AzureKatanaData>().unwrap();
    if let Some(attack) = ws_data.attack.take() {
        for part in attack.slash_parts {
            let unit_rc = part.unit.upgrade().unwrap();
            let unit = unit_rc.borrow();
            response.room_objs_to_remove.push(unit.get_metadata().get_ref());
        }
    }
    response
}

fn exit_room(ctx: &mut WeaponExitRoomContext) -> WeaponExitRoomResponse {
    let ws_data = ctx.ws_data.downcast_mut::<AzureKatanaData>().unwrap();
    let mut response = WeaponExitRoomResponse::new();
    if let Some(attack) = ws_data.attack.take() {
        for part in attack.slash_parts {
            let unit_rc = part.unit.upgrade().unwrap();
            let unit = unit_rc.borrow();
            response.room_objs_to_remove.push(unit.get_metadata().get_ref());
        }
    }
    response
}

fn draw_hud(ctx: &DrawWeaponHudContext) -> DrawWeaponHudResponse {
    let weapon_draw_op = draw_op_annulus(
        (&SLASH_COLOR).into(), 
        (ctx.x + 0.5 * ctx.scale_height, ctx.y + 0.5 * ctx.scale_height), 
        0.4 * ctx.scale_height, 
        0.45 * ctx.scale_height,
    );
    DrawWeaponHudResponse { 
        weapon_draw_op,
        ammo_text: format!("{}", ctx.ammo),
    }
}

fn draw_on_owner(ctx: &DrawWeaponOnOwnerContext) -> DrawWeaponOnOwnerResponse {
    let ws_data = ctx.ws_data.downcast_ref::<AzureKatanaData>().unwrap();
    let mut draw_ops = Vec::new();
    if let Some(ref attack) = ws_data.attack {
        for slash_part in attack.slash_parts.iter() {
            let xlate = Vector::new(ctx.owner_xform.dx as f32, ctx.owner_xform.dy as f32);
            let tri = slash_part.vertexes.map(|v| v + xlate);
            let t = ((ctx.owner_age - slash_part.start_age) / (slash_part.end_age - slash_part.start_age)) as f32;
            let mut color = SLASH_COLOR;
            color.a = 1.0 - 4.0 * f32::powi(t - 0.5, 2);
            draw_ops.push(ctx.draw_ctx.do_tri(color, tri))
        }
    }
    
    let mut color = SLASH_COLOR;
    color.a = 0.01;
    let m2o_angle = if let Some(ref attack) = ws_data.attack {
        attack.start_angle + 0.5 * SLASH_DELTA
    } else {
        f64::atan2(ctx.mouse_y_game_coords - ctx.owner_xform.dy, ctx.mouse_x_game_coords - ctx.owner_xform.dx)
    };
    // invert y (otherwise the annular sector will be drawn y-inverted)
    let m2o_angle = 2.0 * std::f64::consts::PI - m2o_angle;

    let angle1 = m2o_angle - 0.5 * SLASH_DELTA;
    let angle2 = angle1 + SLASH_DELTA;
    let _2pi = 2.0 * std::f64::consts::PI;
    let angle1 = ((angle1 % _2pi) + _2pi) % _2pi;
    let angle2 = ((angle2 % _2pi) + _2pi) % _2pi;
    draw_ops.push(ctx.draw_ctx.do_annular_sector(
        color, 
        Point::new(ctx.owner_xform.dx as f32, ctx.owner_xform.dy as f32), 
        SLASH_PART_RADIUS * 0.95, 
        SLASH_PART_RADIUS, 
        Some((angle1 as f32, angle2 as f32)),
    ));

    draw_ops.push(ctx.draw_ctx.do_annulus(
        SLASH_COLOR, 
        Point::new(ctx.owner_xform.dx as f32, ctx.owner_xform.dy as f32), 
        0.4,
        0.52, 
    ));

    DrawWeaponOnOwnerResponse {
        draw_op: DrawOp::Group(DrawOpGroup::new(draw_ops.into())),
        owner_color: Color::new(0.0, 0.1, 2.0, 1.0),
    }
}