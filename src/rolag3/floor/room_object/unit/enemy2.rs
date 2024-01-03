use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::shape::{Shape, Point}};

use super::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext}, standard_unit_common::TranslateMove};

/* Enemy2 is a green square that continuously moves in the direction of the player.
*/

const SIDE_LEN: f32 = 1.2;

pub struct Enemy2 {
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
}

pub fn new_enemy2(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_square(-SIDE_LEN/2.0, -SIDE_LEN/2.0, SIDE_LEN);
    let us_data = Enemy2 { query_result: None };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Green,
        collision_damage: 10.0,
        hp: 30.0,
        engine_power: 10.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Enemy2>().unwrap();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    if let Some(ref qr) = us_data.query_result {
        match &*qr.borrow() {
            Act1QueryResult::ClosestUnit(v) => {
                if let Some(closest) = v {
                    ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: closest.x - xform.dx, ay: closest.y - xform.dy});
                }
            }
            _ => panic!("unexpected Act1QueryResult. Expected ClosestUnit, got {:?}", qr),
        }
        us_data.query_result = None;
    }

    // x and y in the query shouldn't matter because there's usually one player. I set them anyway in case there are
    // multiple players in the future
    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    let mut response = Act1Response::new();
    us_data.query_result = Some(response.add_query(query));
    response
}

fn draw(ctx: &mut SuDrawContext) {
    let color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(0.1, 0.8, 0.1, 1.0));
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let x = xform.dx as f32 - SIDE_LEN / 2.0;
    let y = xform.dy as f32 - SIDE_LEN / 2.0;
    let w = SIDE_LEN;
    let h = SIDE_LEN;
    let vertexes = [
        Point::new(x, y),
        Point::new(x + w, y),
        Point::new(x + w, y + h),
        Point::new(x, y + h),
    ];
    let dop = ctx.draw_ctx.do_quad_fan( color, vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
}