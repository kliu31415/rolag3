

use std::{rc::Rc, cell::RefCell};

use crate::{geometry::{shape::{Shape, Point}, star::get_star_shape, util::{get_inner_polygon, rotate_polygon, regular_polygon}}, rolag3::floor::{draw::{Color, DrawContext}, rofiz::rofiz_object::Transformation, room_object::{unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuDrawContext, SuAct1Context, StandardUnit1}, standard_unit_common::TranslateMove}, room_object_def::{Team, Act1Response, NewRoomObjectContext, Act1QueryResult, Act1QueryArgs}, damage::DamageColor, projectile::projectile2::{Proj2Shape, Projectile2Builder, Projectile2BuilderReq}}}, util::rng::Prng};

/* SquareRgbStar8 translates towards the player. It moves in the 4 cardinal directions. It periodically stops
   and attacks. It can be one of the RGB colors. The attack depends on its color. can either be:
   -Red: Shoot a laser in one of the 4 cardinal directions
   -Green: Shoot a radial wave of 16 projectiles
   -Blue: Shoot a sequence of ~10 bullets. The bullet angles are perturbed by a gaussian with SD 0.3

   The unit's movement behavior also depends on its color. It'll always move in a cardinal direction towards the player,
   but given a distance to the player (dx, dy), whether it selects the larger or smaller of (dx, dy) depends on the
   unit's color.
*/

const R_OUTER_COLOR: Color = Color::new(0.2, 0.0, 0.0, 1.0);
const R_INNER_COLOR: Color = Color::new(1.0, 0.0, 0.0, 1.0);
const R_PROJ_COLOR: Color = Color::new(5.0, 0.01, 0.01, 1.0);

const G_OUTER_COLOR: Color = Color::new(0.0, 0.1, 0.0, 1.0);
const G_INNER_COLOR: Color = Color::new(0.0, 0.3, 0.0, 1.0);
const G_PROJ_COLOR: Color = Color::new(0.001, 1.5, 0.001, 1.0);

const B_OUTER_COLOR: Color = Color::new(0.01, 0.01, 0.3, 1.0);
const B_INNER_COLOR: Color = Color::new(0.02, 0.02, 0.8, 1.0);
const B_PROJ_COLOR: Color = Color::new(0.05, 0.05, 15.0, 1.0);

const PROJ_RADIUS: f32 = 0.25;

const LASER_PROJ_FIRE_INTERVAL: f64 = 0.0025;
const LASER_PROJ_SPEED: f64 = 180.0;

const RADIAL_BULLET_PROJ_SPEED: f64 = 15.0;

const SEQ_BULLET_PROJ_SPEED: f64 = 17.0;

pub struct SquareRgbStar8 {
    border_vertexes: [Point; 4],
    outer_vertexes: [Point; 4],
    inner_vertexes: [Point; 16],

    outer_color: Color,
    inner_color: Color,
    proj_color: Color,

    move_in_longer_axis_dir: Box<dyn Fn(&mut Prng) -> bool>,
    attack_style: AttackStyle,
    action: Action,
}

enum Action {
    Move {
        xlate_xy: Option<(f64, f64)>,
        force_end_age: f64,
        xlate_query_result: Option<Rc<RefCell<Act1QueryResult>>>,
        early_stop_qr: Option<Rc<RefCell<Act1QueryResult>>>,
    },
    AttackLaser {
        start_age: f64, 
        end_prelude_age: f64, 
        end_age: f64,
        num_proj_fired: i32,
        query_result: Option<Rc<RefCell<Act1QueryResult>>>,
        angle: Option<f64>,
    },
    AttackRadialBulletWave {
        _start_age: f64,
        fire_bullets_age: f64,
        has_fired_bullets: bool,
        end_age: f64,
    },
    AttackBulletSequence {
        _start_age: f64,
        start_fire_age: f64,
        end_age: f64,
        fire_interval: f64,
        num_bullets_fired: i32,
        query_result: Option<Rc<RefCell<Act1QueryResult>>>,
        angle: Option<f64>,
    }
}

enum AttackStyle {
    Laser,
    RadialBulletWave,
    BulletSequence,
}

pub fn new_square_rgb_star8(ctx: &mut NewRoomObjectContext, damage_color: DamageColor, x: f64, y: f64) -> StandardUnit1 {
    let mut border_vertexes: [Point; 4] = regular_polygon(4, 1.2)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let outer_vertexes: [Point; 4] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let inner_vertexes: [Point; 16] = get_star_shape(8, PROJ_RADIUS, 0.5, 0.0)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));

    let (outer_color, inner_color, proj_color) = match damage_color {
        DamageColor::Red => (R_OUTER_COLOR, R_INNER_COLOR, R_PROJ_COLOR),
        DamageColor::Green => (G_OUTER_COLOR, G_INNER_COLOR, G_PROJ_COLOR),
        DamageColor::Blue => (B_OUTER_COLOR, B_INNER_COLOR, B_PROJ_COLOR),
        _ => panic!("unexpected damage_color={:?}", damage_color),
    };

    let attack_style = match damage_color {
        DamageColor::Red => AttackStyle::Laser,
        DamageColor::Green => AttackStyle::RadialBulletWave,
        DamageColor::Blue => AttackStyle::BulletSequence,
        _ => panic!("unexpected damage_color={:?}", damage_color),
    };

    let move_in_longer_axis_dir = match damage_color {
        DamageColor::Red => Box::new(|_: &mut Prng| false) as _,
        DamageColor::Green => Box::new(|_: &mut Prng| true) as _,
        DamageColor::Blue => Box::new(|r: &mut Prng| r.gen_fair_bool()) as _,
        _ => panic!("unexpected damage_color={:?}", damage_color),
    };

    let us_data = SquareRgbStar8 {
        border_vertexes,
        outer_vertexes,
        inner_vertexes,

        outer_color,
        inner_color,
        proj_color,

        move_in_longer_axis_dir,
        attack_style,
        action: Action::Move{xlate_xy: None, force_end_age: 0.5 /*first 0.5s idle*/, xlate_query_result: None, early_stop_qr: None,},
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: 20.0,
        engine_power: 6.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgbStar8>().unwrap();

    let mut response = Act1Response::new();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());

    let mut set_action_to_move = false;
    match &mut us_data.action {
        Action::Move { xlate_xy, xlate_query_result, force_end_age, early_stop_qr } => {
            if let Some(qr) = xlate_query_result.take() {
                let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected xlate_qr={:?}", &*qr.borrow())};
                if let Some(cu) = cu_opt {
                    let dx = cu.x - xform.dx;
                    let dy = cu.y - xform.dy;
                    let move_in_longer_dir = (us_data.move_in_longer_axis_dir)(ctx.act1_ctx.get_rng());
                    let (xlate_x, xlate_y) = if (f64::abs(dx) < f64::abs(dy)) ^ move_in_longer_dir {
                        (f64::signum(dx), 0.0)
                    } else {
                        (0.0, f64::signum(dy))
                    };
                    *xlate_xy = Some((xlate_x, xlate_y));
                }
            }

            if let Some((x, y)) = xlate_xy {
                let movement = TranslateMove::Accelerate { ax: *x, ay: *y };
                ctx.su_ctx.su_common.set_translate_move(movement);
            }

            let mut early_stop = false;
            if let Some(qr) = early_stop_qr {
                let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected early_stop_qr={:?}", &*qr.borrow())};
                if let Some(cu) = cu_opt {
                    let dx = cu.x - xform.dx;
                    let dy = cu.y - xform.dy;
                    early_stop |= f64::abs(dx) < 0.2 || f64::abs(dy) < 0.2;
                    if let Some((xlate_x, xlate_y)) = xlate_xy {
                        early_stop |= *xlate_x * dx < 0.0 || *xlate_y * dy < 0.0;
                    }
                }
            };

            let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
            *early_stop_qr = Some(response.add_query(query));

            if early_stop || unit_age > *force_end_age {
                match us_data.attack_style {
                    AttackStyle::Laser => {
                        let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
                        us_data.action = Action::AttackLaser { 
                            start_age: unit_age,
                            end_prelude_age: unit_age + 1.0, 
                            end_age: unit_age + 2.0, 
                            num_proj_fired: 0, 
                            query_result: Some(response.add_query(query)),
                            angle: None,
                        };
                    }
                    AttackStyle::RadialBulletWave => {
                        us_data.action = Action::AttackRadialBulletWave {
                            _start_age: unit_age,
                            fire_bullets_age: unit_age + 0.5,
                            has_fired_bullets: false,
                            end_age: unit_age + 1.0,
                        };
                    }
                    AttackStyle::BulletSequence => {
                        let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
                        us_data.action = Action::AttackBulletSequence {
                            _start_age: unit_age,
                            start_fire_age: unit_age + 0.5,
                            end_age: unit_age + 1.5,
                            fire_interval: 0.1,
                            num_bullets_fired: 0,
                            query_result: Some(response.add_query(query)),
                            angle: None,
                        };
                    }
                }
            }
        }
        Action::AttackLaser { start_age, end_prelude_age, end_age, num_proj_fired, angle, query_result } => {
            if let Some(qr) = query_result.take() {
                let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected qr={:?}", &*qr.borrow())};
                if let Some(cu) = cu_opt {
                    let a2p = f64::atan2(cu.y - xform.dy, cu.x - xform.dx);
                    *angle = Some(closest_cardinal_angle(a2p));
                }
            }

            if let Some(angle) = angle {
                let since_start = unit_age - *start_age;
                let desired_npf = (since_start / LASER_PROJ_FIRE_INTERVAL) as i32;
                while *num_proj_fired < desired_npf {
                    let since_fired = since_start - *num_proj_fired as f64 * LASER_PROJ_FIRE_INTERVAL;
                    let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                    let (damage, color) = if unit_age < *end_prelude_age {
                        let mut color = us_data.proj_color;
                        color.a = 0.01;
                        (0.0, color)
                    } else {
                        (10.0 * LASER_PROJ_FIRE_INTERVAL, us_data.proj_color)
                    };
                    let proj = Projectile2Builder::new(Projectile2BuilderReq {
                        team: Team::Enemy,
                        damage_color: *ctx.su_ctx.damage_color,
                        damage,
                        owner: self_as_weak,
                        velocity_x: LASER_PROJ_SPEED * f64::cos(*angle),
                        velocity_y: LASER_PROJ_SPEED * f64::sin(*angle),
                        xform: Transformation::new(xform.dx + since_fired * LASER_PROJ_SPEED * f64::cos(*angle), 
                            xform.dy + since_fired * LASER_PROJ_FIRE_INTERVAL * f64::sin(*angle), 
                            0.0),
                        shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: PROJ_RADIUS },
                        color,
                    }).lifespan(2.0)
                        .build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                    response.add_room_obj(Rc::new(RefCell::new(proj)));
                    *num_proj_fired += 1;
                }
            }

            if unit_age > *end_age {
                set_action_to_move = true;
            }
        }
        Action::AttackRadialBulletWave { _start_age: _, fire_bullets_age, has_fired_bullets, end_age } => {
            if !*has_fired_bullets && unit_age > *fire_bullets_age {
                *has_fired_bullets = true;
                let num_bullets = 16;
                for i in 0..num_bullets {
                    let angle = i as f64 / num_bullets as f64 * 2.0 * std::f64::consts::PI;
                    let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                    let proj = Projectile2Builder::new(Projectile2BuilderReq {
                        team: Team::Enemy,
                        damage_color: *ctx.su_ctx.damage_color,
                        damage: 3.0,
                        owner: self_as_weak,
                        velocity_x: RADIAL_BULLET_PROJ_SPEED * f64::cos(angle),
                        velocity_y: RADIAL_BULLET_PROJ_SPEED * f64::sin(angle),
                        xform: Transformation::new(xform.dx, xform.dy, 0.0),
                        shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: PROJ_RADIUS },
                        color: us_data.proj_color,
                    }).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                    response.add_room_obj(Rc::new(RefCell::new(proj)));
                }
            }

            if unit_age > *end_age {
                set_action_to_move = true;
            }
        }
        Action::AttackBulletSequence { _start_age, start_fire_age, end_age, fire_interval, num_bullets_fired, query_result, angle } => {
            if let Some(qr) = query_result.take() {
                let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected qr={:?}", &*qr.borrow())};
                if let Some(cu) = cu_opt {
                    let a2p = f64::atan2(cu.y - xform.dy, cu.x - xform.dx);
                    *angle = Some(closest_cardinal_angle(a2p));
                }
            }
            
            if unit_age > *start_fire_age {
                if let Some(a2p) = angle {
                    let since_start_fire = unit_age - *start_fire_age;
                    let desired_npf = (since_start_fire / *fire_interval) as i32;
                    while *num_bullets_fired < desired_npf {
                        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                        let proj_angle = *a2p + ctx.act1_ctx.get_rng().gen_normal(0.0, 0.3);
                        let proj = Projectile2Builder::new(Projectile2BuilderReq {
                            team: Team::Enemy,
                            damage_color: *ctx.su_ctx.damage_color,
                            damage: 3.0,
                            owner: self_as_weak,
                            velocity_x: SEQ_BULLET_PROJ_SPEED * f64::cos(proj_angle),
                            velocity_y: SEQ_BULLET_PROJ_SPEED * f64::sin(proj_angle),
                            xform: Transformation::new(xform.dx, xform.dy, 0.0),
                            shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: PROJ_RADIUS },
                            color: us_data.proj_color,
                        }).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                        response.add_room_obj(Rc::new(RefCell::new(proj)));
                        *num_bullets_fired += 1;
                    }
                }
            }

            if unit_age > *end_age {
                set_action_to_move = true;
            }
        }
    };

    if set_action_to_move {
        let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
        let query2 = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
        us_data.action = Action::Move { 
            xlate_xy: None,
            xlate_query_result: Some(response.add_query(query)),
            early_stop_qr: Some(response.add_query(query2)),
            force_end_age: unit_age + 5.0,
        };
    }

    response
}

fn closest_cardinal_angle(angle: f64) -> f64 {
    let angle_x = f64::cos(angle);
    let angle_y = f64::sin(angle);
    (0..4)
        .map(|i| i as f64 * std::f64::consts::FRAC_PI_2)
        .max_by(|a, b| {
            let x_a = f64::cos(*a);
            let y_a = f64::sin(*a);
            let x_b = f64::cos(*b);
            let y_b = f64::sin(*b);
            let dot_a = angle_x * x_a + angle_y * y_a;
            let dot_b = angle_x * x_b + angle_y * y_b;
            dot_a.partial_cmp(&dot_b).unwrap()
        }).unwrap()
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgbStar8>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), DrawContext::COLOR_NSU_BORDER);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), us_data.outer_color);
    let inner_color = match us_data.action {
        Action::Move {..} => us_data.inner_color,
        Action::AttackLaser {..} => us_data.proj_color,
        Action::AttackRadialBulletWave { .. } => us_data.proj_color,
        Action::AttackBulletSequence { .. } => us_data.proj_color,
    };
    let inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), inner_color);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let border_vertexes = us_data.border_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let outer_vertexes = us_data.outer_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let inner_vertexes = [Point::new(0.0, 0.0)].iter()
        .chain(us_data.inner_vertexes.iter())
        .chain(us_data.inner_vertexes[..1].iter())
        .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y))
        .collect::<Box<_>>();
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let outer_dop = ctx.draw_ctx.do_quad_fan(outer_color, outer_vertexes);
    let inner_dop = ctx.draw_ctx.do_tri_fan(inner_color, &inner_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, outer_dop, inner_dop])));
}