use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1, RofizObjType}, standard_unit_common::{TranslateMove, RotateMove}}, projectile::projectile2::{Proj2Shape, Projectile2Builder, Projectile2BuilderReq}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point}, util::get_inner_polygon, star::get_star_shape}, util::lerp::lerp_f64};

/* Thinstar3Red jumps from wall to wall. It sprays a wave of 8 projectiles separated by 45 deg every 1.5 while jumping.
*/

const RADIUS: f64 = 1.1;
const MAX_HP: f64 = 20.0;
const BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const OUTER_COLOR: Color = Color::new(0.2, 0.02, 0.02, 1.0);
const INNER_COLOR: Color = Color::new(0.5, 0.02, 0.02, 1.0);
const SHOOT_PROJ_INTERVAL: f64 = 1.5;
const PROJ_COLOR: Color = Color::new(6.0, 0.02, 0.02, 1.0);
const PROJ_RADIUS: f32 = 0.2;

pub struct ThinStar3RedCircle {
    initialized: bool,
    border_vertexes: [Point; 6],
    outer_vertexes: [Point; 6],
    shoot_proj_counter: f64,
    action: Option<WallJumpAction>,
}

struct WallJumpAction {
    prev: (f64, f64),
    next: (f64, f64),
    start_time: f64,
    end_time: f64,
}

pub fn new_thinstar3_red_circle(ctx: &mut NewRoomObjectContext) -> StandardUnit1 {
    let border_vertexes: [Point; 6] = get_star_shape(3, 0.4, RADIUS as f32, 0.0)[..].try_into().unwrap();
    let outer_vertexes: [Point; 6] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(0.0, 0.0, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = ThinStar3RedCircle { 
        initialized: false,
        border_vertexes,
        outer_vertexes,
        shoot_proj_counter: 0.0,
        action: None,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Green,
        collision_damage: 10.0,
        hp: MAX_HP,
        engine_power: 15.0,
        tire_traction: 10.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .angular_power(4.0)
        .angular_traction(5.0)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ThinStar3RedCircle>().unwrap();
    let mut response = Act1Response::new();
    
    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Decelerate);
    if !us_data.initialized {
        us_data.initialized = true;
        let (x, y) = ctx.act1_ctx.sample_wall_xy(RADIUS + 0.5);
        let movement = TranslateMove::SetXY { x, y };
        ctx.su_ctx.su_common.set_translate_move(movement);
    } else {
        let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
        match us_data.action {
            Some(ref action) => {
                let unit_age = ctx.su_ctx.su_common.get_unit_time();
                ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate { atheta: -1.0 });
                if unit_age < action.start_time {
                    // nop
                } else {
                    let lerp_t = f64::min((unit_age - action.start_time) / (action.end_time - action.start_time), 1.0);
                    let new_x = lerp_f64(action.prev.0, action.next.0, lerp_t);
                    let new_y = lerp_f64(action.prev.1, action.next.1, lerp_t);
                    ctx.su_ctx.su_common.set_translate_move(TranslateMove::SetXY { x: new_x, y: new_y });

                    us_data.shoot_proj_counter += ctx.su_ctx.su_common.get_unit_tick_len();
                    if us_data.shoot_proj_counter > SHOOT_PROJ_INTERVAL {
                        us_data.shoot_proj_counter -= SHOOT_PROJ_INTERVAL;
                        if ctx.act1_ctx.is_in_room(xform.dx, xform.dy) {
                            let angle_offset = ctx.act1_ctx.get_randf64() * 2.0 * std::f64::consts::PI;
                            let num_proj = 8;
                            for i in 0..num_proj {
                                let angle = angle_offset + i as f64 / num_proj as f64 * 2.0 * std::f64::consts::PI;
                                let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                                let proj_speed = 10.0;
                                let proj = Projectile2Builder::new(Projectile2BuilderReq {
                                    team: Team::Enemy,
                                    damage_color: DamageColor::Red,
                                    damage: 3.0,
                                    owner: self_as_weak,
                                    velocity_x: proj_speed * f64::cos(angle),
                                    velocity_y: proj_speed * f64::sin(angle),
                                    xform: Transformation::new(xform.dx, xform.dy, 0.0),
                                    shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: PROJ_RADIUS },
                                    color: PROJ_COLOR,
                                }).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                                response.add_room_obj(Rc::new(RefCell::new(proj)));
                            }
                        }
                    }
                }

                if unit_age > action.end_time {
                    us_data.action = None;
                }
            }
            None => {
                let (next_x, next_y) = ctx.act1_ctx.sample_wall_xy(RADIUS + 0.5);
                let dist = f64::hypot(next_x - xform.dx, next_y - xform.dy);
                let prelude_duration = 0.5;
                let start_time = ctx.su_ctx.su_common.get_unit_time() + prelude_duration;
                if ctx.act1_ctx.get_randf64() < 0.3 * ctx.su_ctx.su_common.get_unit_tick_len() {
                    us_data.action = Some(WallJumpAction {
                        prev: (xform.dx, xform.dy),
                        next: (next_x, next_y),
                        start_time,
                        end_time: start_time + 0.125 * dist,
                    });
                }
            }
        }
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ThinStar3RedCircle>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), BORDER_COLOR);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), OUTER_COLOR);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let border_vertexes = us_data.border_vertexes
        .map(|v| v.rotated(xform.dtheta as f32))
        .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let outer_vertexes = us_data.outer_vertexes
        .map(|v| v.rotated(xform.dtheta as f32))
        .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let center = Point::new(xform.dx as f32, xform.dy as f32);
    let tri_fan_vertexes = [center].into_iter()
        .chain(outer_vertexes.iter().cloned())
        .chain(outer_vertexes[..1].iter().cloned())
        .collect::<Box<_>>();
    let outer_dop = ctx.draw_ctx.do_tri_fan(outer_color, &tri_fan_vertexes);
    let lerp_t = f64::min(1.0, 4.0 * f64::min(us_data.shoot_proj_counter, SHOOT_PROJ_INTERVAL - us_data.shoot_proj_counter)) as f32;
    let inner_color = Color::lerp(PROJ_COLOR, INNER_COLOR, lerp_t);
    let inner_dop = ctx.draw_ctx.do_circle(inner_color, Point::new(xform.dx as f32, xform.dy as f32), PROJ_RADIUS);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, outer_dop, inner_dop])));
}