use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{draw::{Color, DrawContext}, room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, HandleCollisionResponse}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, HandleCollisionLogic, SuAct1Context, SuDrawContext, SuHandleCollisionContext}, standard_unit_common::TranslateMove}, damage::DamageColor, projectile::projectile2::{Proj2Shape, Projectile2BuilderReq, Projectile2Builder}}, rofiz::rofiz_object::Transformation}, geometry::{shape::{Shape, Point}, util::{regular_polygon, get_inner_polygon}}};

/* BlueHexBlueHex randomly translates in one of 6 directions and periodically shoots a wave of 6 projectiles rotate
   in a circle and move in a random direction
*/

const OUTER_COLOR: Color = Color::new(0.0, 0.0, 0.2, 1.0);
const INNER_COLOR: Color = Color::new(0.0, 0.0, 1.0, 1.0);
const PROJ_COLOR: Color = Color::new(0.2, 0.2, 14.0, 1.0);

pub struct HexagonBlueHexagon {
    spit_projectile_start: Option<SpitProjectileInfo>,
    border_vertexes: [Point; 6],
    outer_vertexes: [Point; 6],
    inner_vertexes: [Point; 6],
    translate_dir: i64,
    should_reset_velocity: bool,
}

struct SpitProjectileInfo {
    start: f64,
    proj_spit: bool,
}

pub fn new_hexagon_blue_hexagon(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let border_vertexes: [Point; 6] = regular_polygon(6, 0.9)[..].try_into().unwrap();
    let outer_vertexes: [Point; 6] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let inner_vertexes: [Point; 6] = get_inner_polygon(0.55, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = HexagonBlueHexagon {
        spit_projectile_start: None,
        border_vertexes,
        outer_vertexes,
        inner_vertexes,
        translate_dir: ctx.get_randi64(0..6),
        should_reset_velocity: false,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Blue,
        hp: 20.0,
        engine_power: 15.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<HexagonBlueHexagon>().unwrap();

    let mut response = Act1Response::new();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();

    match us_data.spit_projectile_start {
        Some(ref mut sps) => {
            if !sps.proj_spit && ctx.act1_ctx.get_room_time() - sps.start > 0.5 {
                sps.proj_spit = true;
                let main_dir_angle = ctx.act1_ctx.get_randf64() * 2.0 * std::f64::consts::PI;
                let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
                let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                for i in 0..6 {
                    let angle = (i as f64) * 1.0/3.0 * std::f64::consts::PI;
                    let shape = Proj2Shape::TriFan {
                        center: Point::new(0.0, 0.0), 
                        vertexes: Box::new(us_data.inner_vertexes),
                    };
                    let nef_position_fn = move |age: f64| -> (f64, f64) {
                        let radians_per_s = 3.0;
                        let radial_expand_velocity = 5.0;
                        let expand1_until = 1.0;
                        let rotate1_until = 2.0;
                        let (x, y) = if age < expand1_until {
                            (radial_expand_velocity * age * f64::cos(angle), 
                             radial_expand_velocity * age * f64::sin(angle))
                        } else if age < rotate1_until {
                            let r = radial_expand_velocity * expand1_until;
                            let t = age - expand1_until;
                            (r * f64::cos(angle + radians_per_s * t), 
                             r * f64::sin(angle + radians_per_s * t))
                        } else {
                            let r = radial_expand_velocity * expand1_until;
                            let t = age - expand1_until;
                            let (x, y) = (r * f64::cos(angle + radians_per_s * t), 
                                          r * f64::sin(angle + radians_per_s * t));

                            let main_dir_velocity = 10.0;
                            (x + (age - rotate1_until) * main_dir_velocity * f64::cos(main_dir_angle),
                             y + (age - rotate1_until) * main_dir_velocity * f64::sin(main_dir_angle))
                        };
                        (x, y)
                    };
                    let proj = Projectile2Builder::new(
                        Projectile2BuilderReq {
                            team: Team::Enemy,
                            damage_color: DamageColor::Blue,
                            damage: 3.0,
                            owner: self_as_weak.clone(),
                            velocity_x: 0.0,
                            velocity_y: 0.0,
                            xform,
                            shape,
                            color: PROJ_COLOR,
                        }
                    ).nef_position_fn(Box::new(nef_position_fn))
                        .build(&mut nfo_ctx);
                    response.add_room_obj(Rc::new(RefCell::new(proj)));
                }
            }
            if ctx.act1_ctx.get_room_time() - sps.start > 1.0 {
                us_data.spit_projectile_start = None;
            }
        },
        None => if ctx.act1_ctx.get_randf64() < tick_len {
            us_data.translate_dir = ctx.act1_ctx.get_randi64(0..6);
            us_data.spit_projectile_start = Some(SpitProjectileInfo { start: ctx.act1_ctx.get_room_time(), proj_spit: false});
        },
    };

    match us_data.spit_projectile_start {
        Some(_) => {
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Decelerate);
        }
        None => {
            let theta = (us_data.translate_dir as f64 + 0.5) * 1.0/3.0 * std::f64::consts::PI;
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: f64::cos(theta), ay: f64::sin(theta)});
        }
    }

    if us_data.should_reset_velocity {
        us_data.should_reset_velocity = false;
        ctx.su_ctx.su_common.set_translate_move(TranslateMove::ResetVelocity);
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<HexagonBlueHexagon>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(OUTER_COLOR);
    let inner_color = match us_data.spit_projectile_start {
        Some(ref x) => Color::lerp(PROJ_COLOR, INNER_COLOR, 2.0 * f64::abs(0.5 - (ctx.draw_ctx.get_room_time() - x.start)) as f32),
        None => INNER_COLOR,
    };
    let inner_color = ctx.su_ctx.su_common.get_draw_color(inner_color);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let border_vertexes = us_data.border_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let outer_vertexes = us_data.outer_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let inner_vertexes = us_data.inner_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let outer_dop = ctx.draw_ctx.do_thick_border(outer_color, &outer_vertexes, &inner_vertexes);
    let inner_dop = ctx.draw_ctx.do_tri_fan(inner_color, &inner_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, outer_dop, inner_dop])));
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<HexagonBlueHexagon>().unwrap();
    if !ctx.hc_ctx.is_other_spectral() {
        us_data.should_reset_velocity = true;
        us_data.translate_dir = ctx.hc_ctx.get_randi64(0..6);
    }
    HandleCollisionResponse::new()
}