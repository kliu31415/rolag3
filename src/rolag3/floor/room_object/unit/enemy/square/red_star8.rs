

use std::{rc::Rc, cell::RefCell};

use crate::{geometry::{shape::{Shape, Point}, star::get_star_shape, util::{get_inner_polygon, rotate_polygon, regular_polygon}}, rolag3::floor::{draw::{Color, DrawContext}, rofiz::rofiz_object::Transformation, room_object::{unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuDrawContext, SuAct1Context, StandardUnit1}, standard_unit_common::TranslateMove}, room_object_def::{Team, Act1Response, NewRoomObjectContext, Act1QueryResult, Act1QueryArgs}, damage::DamageColor, projectile::projectile2::{Proj2Shape, Projectile2Builder, Projectile2BuilderReq}}}};

const OUTER_COLOR: Color = Color::new(0.2, 0.0, 0.0, 1.0);
const INNER_COLOR: Color = Color::new(1.0, 0.0, 0.0, 1.0);
const PROJ_COLOR: Color = Color::new(5.0, 0.2, 0.2, 1.0);

const PROJ_FIRE_INTERVAL: f64 = 0.0025;
const PROJ_SPEED: f64 = 180.0;
const PROJ_RADIUS: f32 = 0.25;

/* RedSquareRedStar8 translates towards the player. It moves in the 4 cardinal directions.
*/

pub struct RedSquareRedStar8 {
    action: Action,
    border_vertexes: [Point; 4],
    outer_vertexes: [Point; 4],
    inner_vertexes: [Point; 16],
}

enum Action {
    Move {
        xlate_angle: Option<f64>,
        end_age: f64,
        query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    },
    Attack {
        start_age: f64, 
        end_prelude_age: f64, 
        end_age: f64,
        num_proj_fired: i32,
        query_result: Option<Rc<RefCell<Act1QueryResult>>>,
        angle: Option<f64>,
    }
}

pub fn new_square_red_star8(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let mut border_vertexes: [Point; 4] = regular_polygon(4, 1.2)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let outer_vertexes: [Point; 4] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let inner_vertexes: [Point; 16] = get_star_shape(8, PROJ_RADIUS, 0.5, 0.0)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = RedSquareRedStar8 {
        action: Action::Move{xlate_angle: None, end_age: 0.5 /*first 0.5s idle*/, query_result: None},
        border_vertexes,
        outer_vertexes,
        inner_vertexes,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Red,
        hp: 20.0,
        engine_power: 12.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RedSquareRedStar8>().unwrap();

    let mut response = Act1Response::new();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());

    match &mut us_data.action {
        Action::Move { xlate_angle, query_result, end_age } => {
            if let Some(qr) = query_result.take() {
                let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected qr={:?}", &*qr.borrow())};
                if let Some(cu) = cu_opt {
                    let a2p = f64::atan2(cu.y - xform.dy, cu.x - xform.dx);
                    *xlate_angle = Some(closest_cardinal_angle(a2p));
                }
            }

            if let Some(xlate_angle) = xlate_angle {
                let movement = TranslateMove::Accelerate { ax: f64::cos(*xlate_angle), ay: f64::sin(*xlate_angle) };
                ctx.su_ctx.su_common.set_translate_move(movement);
            }

            if unit_age > *end_age {
                let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
                us_data.action = Action::Attack { 
                    start_age: unit_age,
                    end_prelude_age: unit_age + 1.0, 
                    end_age: unit_age + 2.0, 
                    num_proj_fired: 0, 
                    query_result: Some(response.add_query(query)),
                    angle: None,
                };
            }
        }
        Action::Attack { start_age, end_prelude_age, end_age, num_proj_fired, angle, query_result } => {
            if let Some(qr) = query_result.take() {
                let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected qr={:?}", &*qr.borrow())};
                if let Some(cu) = cu_opt {
                    let a2p = f64::atan2(cu.y - xform.dy, cu.x - xform.dx);
                    *angle = Some(closest_cardinal_angle(a2p));
                }
            }

            if let Some(angle) = angle {
                let since_start = unit_age - *start_age;
                let desired_npf = (since_start / PROJ_FIRE_INTERVAL) as i32;
                while *num_proj_fired < desired_npf {
                    let since_fired = since_start - *num_proj_fired as f64 * PROJ_FIRE_INTERVAL;
                    let self_as_weak = ctx.act1_ctx.get_self_as_weak();
                    let (damage, color) = if unit_age < *end_prelude_age {
                        let mut color = PROJ_COLOR;
                        color.a = 0.01;
                        (0.0, color)
                    } else {
                        (10.0 * PROJ_FIRE_INTERVAL, PROJ_COLOR)
                    };
                    let proj = Projectile2Builder::new(Projectile2BuilderReq {
                        team: Team::Enemy,
                        damage_color: DamageColor::Red,
                        damage,
                        owner: self_as_weak,
                        velocity_x: PROJ_SPEED * f64::cos(*angle),
                        velocity_y: PROJ_SPEED * f64::sin(*angle),
                        xform: Transformation::new(xform.dx + since_fired * PROJ_SPEED * f64::cos(*angle), 
                            xform.dy + since_fired * PROJ_FIRE_INTERVAL * f64::sin(*angle), 
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
                let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
                us_data.action = Action::Move { 
                    xlate_angle: None,
                    query_result: Some(response.add_query(query)),
                    end_age: unit_age + 0.5 + ctx.act1_ctx.get_randf64(),
                };
            }
        }
    };

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
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RedSquareRedStar8>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), DrawContext::COLOR_NSU_BORDER);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), OUTER_COLOR);
    let inner_color = match us_data.action {
        Action::Move {..} => INNER_COLOR,
        Action::Attack {..} => PROJ_COLOR,
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