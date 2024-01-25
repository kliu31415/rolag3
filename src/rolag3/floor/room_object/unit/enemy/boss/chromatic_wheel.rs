use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, Act1QueryArgs, Act1QueryResult}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, AsBossHpLogic, SuAct1Context, SuDrawContext}, standard_unit_common::RotateMove}, damage::DamageColor, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::shape::{Shape, Point, Vector}};

/* ChromaticWheel stays in the center of the room and doesn't translate. It rotates. It has 12 subcircles along the
   edges, alternating between Red and Blue. It has one large Green subcircle in the center. Attacks (multiple can
   happen at once) happen periodically:
   - Red subcircles fire lasers radially outwards
     - At first, only half of redcircles do this. When the boss's HP drops low enough, all red subcircles do this
   - Blue subcircles fire lasers toward the player after a short prelude
     - At first, no blue subcircles do this. More and more blue subcircles do this as the boss loses hp.
   - The green subcircle releases a radial wave of projectiles
 */

const MAX_HP: f64 = 600.0;
const STAGE_1_HP_THRESHOLD: f64 = 300.0;
const RADIUS: f32 = 2.5;
const CENTRAL_SUBCIRCLE_RADIUS: f32 = 0.6;
const OUTER_SUBCIRCLE_RADIUS: f32 = 0.3;
const NUM_OUTER_CIRCLES: usize = 12; // alternates between Red and Blue color
const BORDER_THICKNESS: f32 = 0.1;

const MAIN_INNER_COLOR: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const R_SUBCIRCLE_COLOR: Color = Color::new(0.5, 0.001, 0.001, 1.0);
const G_SUBCIRCLE_COLOR: Color = Color::new(0.001, 0.5, 0.001, 1.0);
const B_SUBCIRCLE_COLOR: Color = Color::new(0.001, 0.001, 0.5, 1.0);

const B_FIRE_INTERVAL: f64 = 0.0025;
const B_PROJ_RADIUS: f32 = 0.25;
const B_PROJ_SPEED: f64 = 180.0;
const B_PROJ_COLOR: Color = Color::new(0.05, 0.05, 15.0, 1.0);

const R_FIRE_INTERVAL: f64 = 0.0025;
const R_PROJ_RADIUS: f32 = 0.25;
const R_PROJ_SPEED: f64 = 180.0;
const R_PROJ_COLOR: Color = Color::new(5.5, 0.01, 0.01, 1.0);

const G_PROJ_COLOR: Color = Color::new(0.001, 1.5, 0.001, 1.0);
const G_PROJ_RADIUS: f32 = 0.3;

struct ChromaticWheel {
    rotate_dir: Option<f64>,
    last_green_wave_at: f64,
    next_green_wave_at: f64,
    red_subcircle_info: [RedSubcircleInfo; NUM_OUTER_CIRCLES / 2],
    blue_subcircle_info: [BlueSubcircleInfo; NUM_OUTER_CIRCLES / 2],
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
}

struct RedSubcircleInfo {
    center_r: f64,
    center_theta: f64,

    fi: Option<RedSubcircleFireInfo>,
}

struct RedSubcircleFireInfo {
    start_prelude_age: f64,
    start_fire_age: f64,
    num_proj_fired: i32,
}

struct BlueSubcircleInfo {
    center_r: f64,
    center_theta: f64,
    active: bool,

    num_proj_fired: i32,
    fire_angle: Option<f64>,

    start_prelude_age: f64,
    start_fire_age: f64,
    end_age: f64,
}

pub fn new_boss_chromatic_wheel(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), RADIUS);
    let us_data = ChromaticWheel {
        query_result: None,
        rotate_dir: None,
        last_green_wave_at: 0.0,
        next_green_wave_at: 1.0,
        red_subcircle_info: std::array::from_fn(|i| RedSubcircleInfo {
            center_r: (RADIUS - OUTER_SUBCIRCLE_RADIUS - BORDER_THICKNESS - 0.1) as f64,
            center_theta: (2 * i + 1) as f64 / NUM_OUTER_CIRCLES as f64 * 2.0 * std::f64::consts::PI + xform.dtheta,
            fi: None,
        }),
        blue_subcircle_info: std::array::from_fn(|i| BlueSubcircleInfo {
            center_r: (RADIUS - OUTER_SUBCIRCLE_RADIUS - BORDER_THICKNESS - 0.1) as f64,
            center_theta: (2 * i) as f64 / NUM_OUTER_CIRCLES as f64 * 2.0 * std::f64::consts::PI + xform.dtheta,
            active: false,
            num_proj_fired: 0,
            fire_angle: None,
            start_prelude_age: i as f64 * 0.25,
            start_fire_age: i as f64 * 0.25 + 0.75,
            end_age: i as f64 * 0.25 + 1.0,
        }),
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Silver,
        hp: MAX_HP,
        engine_power: 0.0,
        tire_traction: 0.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .us_data(Box::new(us_data))
        .as_boss_hp_logic(AsBossHpLogic::Basic)
        .hitbox(xform, shape)
        .angular_power(1.0)
        .angular_traction(0.5)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ChromaticWheel>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());

    let hp_pct = ctx.su_ctx.su_common.get_cur_hp() / ctx.su_ctx.su_common.get_max_hp();
    let mut iter = 0;
    while unit_age >= us_data.next_green_wave_at {
        iter += 1;
        assert!(iter < 1000);

        us_data.last_green_wave_at = us_data.next_green_wave_at;
        us_data.next_green_wave_at += 1.0 + 1.0 * hp_pct;

        let proj_speed = 5.0 + 5.0 * (1.0 - hp_pct);
        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
        let num_proj = 64;
        for i in 0..num_proj {
            let angle = i as f64 / num_proj as f64 * 2.0 * std::f64::consts::PI;
            let proj = Projectile2Builder::new(
                Projectile2BuilderReq {
                    team: Team::Enemy,
                    damage_color: DamageColor::Green,
                    damage: 3.0,
                    owner: self_as_weak.clone(),
                    velocity_x: proj_speed * f64::cos(angle),
                    velocity_y: proj_speed * f64::sin(angle),
                    xform,
                    shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: G_PROJ_RADIUS },
                    color: G_PROJ_COLOR,
                }
            ).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
    }

    let num_blue_subcircles = us_data.blue_subcircle_info.len();
    for (i, bs) in us_data.blue_subcircle_info.iter_mut().enumerate() {
        if bs.active && unit_age > bs.start_prelude_age {
            let center_x = xform.dx + bs.center_r * f64::cos(bs.center_theta + xform.dtheta);
            let center_y = xform.dx + bs.center_r * f64::sin(bs.center_theta + xform.dtheta);

            if bs.fire_angle.is_none() {
                bs.fire_angle = Some(f64::atan(2.0 * std::f64::consts::PI * ctx.act1_ctx.get_randf64()));
                if let Some(ref qr) = us_data.query_result {
                    let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected Act1QueryResult. Expected ClosestUnit, got {:?}", qr)};
                    if let Some(cu) = cu_opt {
                        bs.fire_angle = Some(f64::atan2(cu.y - center_y, cu.x - center_x));
                    }
                }
            }

            let since_start = unit_age - bs.start_prelude_age;
            let desired_npf = (since_start / B_FIRE_INTERVAL) as i32;
            while bs.num_proj_fired < desired_npf {
                let fired_time_ago = since_start - bs.num_proj_fired as f64 * B_FIRE_INTERVAL;
                let start_offset = fired_time_ago * B_PROJ_SPEED;
                let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                let proj_xform = Transformation::new(
                    center_x + start_offset * f64::cos(bs.fire_angle.unwrap()),
                    center_y + start_offset * f64::sin(bs.fire_angle.unwrap()),
                    0.0,
                );
                let (damage, draw_color) = if unit_age < bs.start_fire_age {
                    let mut color = B_PROJ_COLOR;
                    color.a = 0.01;
                    (0.0, color)
                } else {
                    (10.0 * B_FIRE_INTERVAL, B_PROJ_COLOR)
                };

                let proj = Projectile2Builder::new(
                    Projectile2BuilderReq{
                        team: Team::Enemy,
                        damage_color: DamageColor::Blue,
                        damage,
                        owner: self_as_weak,
                        velocity_x: B_PROJ_SPEED * f64::cos(bs.fire_angle.unwrap()),
                        velocity_y: B_PROJ_SPEED * f64::sin(bs.fire_angle.unwrap()),
                        xform: proj_xform,
                        shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: B_PROJ_RADIUS },
                        color: draw_color,
                    }
                ).lifespan(2.0)
                    .build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                response.add_room_obj(Rc::new(RefCell::new(proj)));
                bs.num_proj_fired += 1;
            }
        }

        if unit_age > bs.end_age {
            let hp_pct = ctx.su_ctx.su_common.get_cur_hp() / ctx.su_ctx.su_common.get_max_hp();
            let a = 2 * i + num_blue_subcircles;
            let b = 3 * num_blue_subcircles;
            if hp_pct < a as f64 / b as f64 {
                bs.active = true;
            }
            let prev_end_age = bs.end_age;
            bs.num_proj_fired = 0;
            bs.fire_angle = None;
            bs.start_prelude_age = prev_end_age + 2.0;
            bs.start_fire_age = prev_end_age + 2.75;
            bs.end_age = prev_end_age + 3.0;
        }
    }

    for (i, rs) in us_data.red_subcircle_info.iter_mut().enumerate() {
        if rs.fi.is_none() {
            if i % 2 == 0 || (i%2==1 && ctx.su_ctx.su_common.get_cur_hp() < STAGE_1_HP_THRESHOLD) {
                rs.fi = Some(RedSubcircleFireInfo {
                    start_prelude_age: unit_age,
                    start_fire_age: unit_age + 1.0,
                    num_proj_fired: 0,
                })
            }
        }
        if let Some(ref mut fi) = rs.fi {
            let center_x = xform.dx + rs.center_r * f64::cos(rs.center_theta + xform.dtheta);
            let center_y = xform.dx + rs.center_r * f64::sin(rs.center_theta + xform.dtheta);
            if unit_age > fi.start_prelude_age {
                let since_start = unit_age - fi.start_prelude_age;
                let desired_npf = (since_start / R_FIRE_INTERVAL) as i32;
                while fi.num_proj_fired < desired_npf {
                    let fired_time_ago = since_start - fi.num_proj_fired as f64 * R_FIRE_INTERVAL;
                    let start_offset = fired_time_ago * R_PROJ_SPEED;
                    let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                    let proj_xform = Transformation::new(
                        center_x + start_offset * f64::cos(rs.center_theta + xform.dtheta),
                        center_y + start_offset * f64::sin(rs.center_theta + xform.dtheta),
                        0.0,
                    );
                    let (damage, draw_color) = if unit_age < fi.start_fire_age {
                        let mut color = R_PROJ_COLOR;
                        color.a = 0.01;
                        (0.0, color)
                    } else {
                        (10.0 * R_FIRE_INTERVAL, R_PROJ_COLOR)
                    };
                    let proj = Projectile2Builder::new(
                        Projectile2BuilderReq{
                            team: Team::Enemy,
                            damage_color: DamageColor::Red,
                            damage,
                            owner: self_as_weak,
                            velocity_x: R_PROJ_SPEED * f64::cos(rs.center_theta + xform.dtheta),
                            velocity_y: R_PROJ_SPEED * f64::sin(rs.center_theta + xform.dtheta),
                            xform: proj_xform,
                            shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: R_PROJ_RADIUS },
                            color: draw_color,
                        }
                    ).lifespan(2.0)
                        .build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                    response.add_room_obj(Rc::new(RefCell::new(proj)));
                    fi.num_proj_fired += 1;
                }
            }
        }
    }

    if ctx.act1_ctx.get_randf64() < 0.25 * ctx.su_ctx.su_common.get_unit_tick_len() {
        let randv = ctx.act1_ctx.get_randf64();
        us_data.rotate_dir = if randv < 0.3 {
            Some(-1.0)
        } else if randv < 0.6 {
            Some(1.0)
        } else {
            None
        };
    }
    if let Some(rotate_dir) = us_data.rotate_dir { 
        ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate {atheta: rotate_dir});
    } else {
        ctx.su_ctx.su_common.set_rotate_move(RotateMove::Decelerate);
    }

    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    us_data.query_result = Some(response.add_query(query));

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ChromaticWheel>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let mut dops = Vec::new();
    dops.push(ctx.draw_ctx.do_concentric_circle(
        ctx.su_ctx.su_common.get_draw_color(MAIN_INNER_COLOR), 
        DrawContext::COLOR_NSU_BORDER, 
        Point::new(xform.dx as f32, xform.dy as f32),
        RADIUS - BORDER_THICKNESS,
        RADIUS,
    ));

    let vec_xform = Vector::new(xform.dx as f32, xform.dy as f32);
    for rs in us_data.red_subcircle_info.iter() {
        let r = rs.center_r as f32;
        let theta = (rs.center_theta + xform.dtheta) as f32;
        let center = Point::new(r * f32::cos(theta), r * f32::sin(theta)) + vec_xform;
        let color = if rs.fi.is_some() {
            R_PROJ_COLOR
        } else {
            R_SUBCIRCLE_COLOR
        };
        let color = ctx.su_ctx.su_common.get_draw_color(color);
        dops.push(ctx.draw_ctx.do_circle(color, center, OUTER_SUBCIRCLE_RADIUS));
    }
    for bs in us_data.blue_subcircle_info.iter() {
        let r = bs.center_r as f32;
        let theta = (bs.center_theta + xform.dtheta) as f32;
        let center = Point::new(r * f32::cos(theta), r * f32::sin(theta)) + vec_xform;
        let color = if unit_age > bs.start_prelude_age {
            B_PROJ_COLOR
        } else {
            B_SUBCIRCLE_COLOR
        };
        let color = ctx.su_ctx.su_common.get_draw_color(color);
        dops.push(ctx.draw_ctx.do_circle(color, center, OUTER_SUBCIRCLE_RADIUS));
    }

    let lerp_t = f64::min(1.0, 4.0 * f64::min(us_data.next_green_wave_at - unit_age, unit_age - us_data.last_green_wave_at));
    let color = Color::lerp(G_PROJ_COLOR, G_SUBCIRCLE_COLOR, lerp_t as f32);
    let color = ctx.su_ctx.su_common.get_draw_color(color);
    dops.push(ctx.draw_ctx.do_circle(color, Point::new(xform.dx as f32, xform.dy as f32), CENTRAL_SUBCIRCLE_RADIUS));

    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(dops.into()));
}