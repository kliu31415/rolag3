use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, Act1QueryResult, Act1QueryArgs}, damage::DamageColor}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::shape::Shape};

use super::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, StandardUnit1, SuAct1Context, SuDrawContext}, standard_unit_common::TranslateMove};

/* Enemy3 is a red square that periodically charges in the direction of the player.
*/

const SIDE_LEN: f32 = 1.8;

pub struct Enemy3Data {
    move_charge: Option<MoveCharge>,
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
}

pub fn new_enemy3(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_square(-SIDE_LEN/2.0, -SIDE_LEN/2.0, SIDE_LEN);
    let us_data = Enemy3Data {
        move_charge: None,
        query_result: None,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Red,
        hp: 30.0,
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
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Enemy3Data>().unwrap();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());

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
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Enemy3Data>().unwrap();
    let cx = Color::new(2.0, 0.0, 0.0, 1.0);
    let cy = Color::new(5.0, 0.5, 0.5, 1.0);
    let color = match us_data.move_charge {
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
            Color::lerp(cx, cy, a)
        },
        None => cx,
    };
    let color = ctx.su_ctx.su_common.get_draw_color(color);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let x = xform.dx as f32 - SIDE_LEN / 2.0;
    let y = xform.dy as f32 - SIDE_LEN / 2.0;
    let s = SIDE_LEN;
    let dop = ctx.draw_ctx.do_rect(color, x, y, s, s);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
}