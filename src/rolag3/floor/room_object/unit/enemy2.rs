use crate::{rolag3::floor::{room_object::room_object_def::{NewRoomObjectContext, Act1Response, Team}, rofiz::rofiz_object::Transformation, draw::{Color, FloorDrawCoordinate, DrawContext}}, geometry::shape::Shape};

use super::standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext};

const SIDE_LEN: f32 = 1.2;

pub struct Enemy2 {

}

pub fn new_enemy2(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_square(-SIDE_LEN/2.0, -SIDE_LEN/2.0, SIDE_LEN);
    let us_data = Enemy2 { };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        hp: 30.0,
        engine_power: 40.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let response = Act1Response::new();
    let tick_len = ctx.act1_ctx.get_tick_length();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let player_xy = ctx.act1_ctx.get_team_closest_location(Team::Player);
    if let Some(xy) = player_xy {
        ctx.su_ctx.su_common.accelerate_ro_xy(tick_len, xy.x - xform.dx, xy.y - xform.dy);
    }
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
        FloorDrawCoordinate::new(x, y),
        FloorDrawCoordinate::new(x + w, y),
        FloorDrawCoordinate::new(x + w, y + h),
        FloorDrawCoordinate::new(x, y + h),
    ];
    let dop = ctx.draw_ctx.do_quad( color, vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
}