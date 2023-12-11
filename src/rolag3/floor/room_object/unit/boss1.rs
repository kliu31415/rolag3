use crate::{rolag3::floor::{room_object::room_object_def::{NewRoomObjectContext, Act1Response, Team}, rofiz::rofiz_object::Transformation, draw::{Color, FloorDrawCoordinate, DrawContext}}, geometry::{star::get_star_shape, shape::{Shape, f32pairs_to_shape}}};

use super::standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext};

struct Boss1 {

}

pub fn new_boss1(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(f32pairs_to_shape(get_star_shape(5, 2.0, 3.0, 0.0)));
    let us_data = Boss1 { };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        hp: 1e5,
        engine_power: 20.0,
        tire_traction: 20.0,
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
    let color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(10.0, 0.0, 0.0, 1.0));
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let shape = ctx.draw_ctx.get_rofiz().get_movable_object_xformed_shape(ctx.su_ctx.su_common.get_ro_ref());
    if let Shape::Polygon(p) = shape {
        let mut vertexes = Vec::new();
        vertexes.push(FloorDrawCoordinate::new(xform.dx as f32, xform.dy as f32));
        for v in p.vertexes.iter().chain(std::iter::once(&p.vertexes[0])) {
            vertexes.push(FloorDrawCoordinate::new(v.x, v.y));
        }
        let dop = ctx.draw_ctx.do_tri_fan(color, vertexes.into_boxed_slice());
        ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
    } else {
        panic!("shape is not polygon");
    }
}