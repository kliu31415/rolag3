use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{Act1QueryResult, NewRoomObjectContext, Team, Act1Response, Act1QueryArgs}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext}, standard_unit_common::TranslateMove}, damage::DamageColor}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point}, util::{rotate_polygon, regular_polygon, get_inner_polygon}}};

/* SquareRed periodically charges at the player
 */

const INNER_COLOR_INACTIVE: Color = Color::new(2.0, 0.0, 0.0, 1.0);
const INNER_COLOR_ACTIVE: Color = Color::new(5.0, 0.1, 0.1, 1.0);

struct SquareRed {
    move_charge: Option<MoveCharge>,
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    border_vertexes: [Point; 4],
    inner_vertexes: [Point; 4],
}

pub fn new_square_red(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let mut border_vertexes: [Point; 4] = regular_polygon(4, 1.2)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let inner_vertexes: [Point; 4] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = SquareRed {
        move_charge: None,
        query_result: None,
        border_vertexes,
        inner_vertexes,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Red,
        hp: 20.0,
        engine_power: 20.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

struct MoveCharge {
    started_at: f64,
    velocity_x: f64,
    velocity_y: f64,
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRed>().unwrap();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());

    if let Some(ref qr) = us_data.query_result {
        match &*qr.borrow() {
            Act1QueryResult::ClosestUnit(v) => {
                if let Some(closest) = v {
                    let theta = f64::atan2(closest.y - xform.dy, closest.x - xform.dx);
                    let velocity_x = f64::cos(theta);
                    let velocity_y = f64::sin(theta);
                    us_data.move_charge = Some(MoveCharge { started_at: ctx.act1_ctx.get_room_time(), velocity_x, velocity_y});
                }
            }
            _ => panic!("unexpected Act1QueryResult. Expected ClosestUnit, got {:?}", qr),
        }
        us_data.query_result = None;
    }

    match us_data.move_charge {
        Some(ref mc) => {
            let since_start = ctx.act1_ctx.get_room_time() - mc.started_at;
            if since_start > 0.5 {
                ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate{ax: mc.velocity_x, ay: mc.velocity_y});
                if since_start > 4.0 {
                    us_data.move_charge = None;
                }
            }
        },
        None => {
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Decelerate);
            if ctx.act1_ctx.get_randf64() < tick_len {
                // x and y in the query shouldn't matter because there's usually one player. I set them anyway in case there are
                // multiple players in the future
                let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
                us_data.query_result = Some(response.add_query(query));
            }
        },
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRed>().unwrap();
    let inner_color = match us_data.move_charge {
        Some(ref v) => {
            let since_start = (ctx.draw_ctx.get_room_time() - v.started_at) as f32;
            assert!(since_start>=0.0 && since_start<=4.0);
            let a: f32;
            if since_start < 0.5 {
                a = 2.0 * since_start;
            } else if since_start < 3.5 {
                a = 1.0;
            } else {
                a = 2.0 * (4.0 - since_start);
            }
            Color::lerp(INNER_COLOR_INACTIVE, INNER_COLOR_ACTIVE, a)
        },
        None => INNER_COLOR_INACTIVE,
    };
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(inner_color);
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let border_vertexes = us_data.border_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let inner_vertexes = us_data.inner_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &inner_vertexes);
    let inner_dop = ctx.draw_ctx.do_quad_fan(inner_color, inner_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, inner_dop])));
}