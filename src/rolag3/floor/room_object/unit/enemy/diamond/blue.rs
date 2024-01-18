use std::{rc::Rc, cell::RefCell};

use crate::{geometry::{shape::{Point, Shape, Vector}, util::get_inner_polygon}, rolag3::floor::{draw::{Color, DrawContext}, room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, Act1QueryResult, Act1QueryArgs}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, RofizObjType}, standard_unit_common::{RotateMove, TranslateMove}}, damage::DamageColor}, rofiz::rofiz_object::Transformation}};

/* DiamondBlue is a diamond composed of two equilateral-like triangles. It rotates towards the player and
   moves in the direction of the tips. It's spawned by BigCircleBlueDiamond.
 */

pub const BORDER_VERTEXES: [Point; 4] = [Point::new(0.8, 0.0), Point::new(0.0, 0.5), Point::new(-0.8, 0.0), Point::new(0.0, -0.5)];
pub const INNER_COLOR: Color = Color::new(0.03, 0.03, 5.0, 1.0);

struct DiamondBlue {
    inner_vertexes: [Point; 4],
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
}

pub fn new_diamond_blue(ctx: &mut NewRoomObjectContext, x: f64, y: f64, theta: f64) -> StandardUnit1 {
    let inner_vertexes: [Point; 4] = get_inner_polygon(0.1, &BORDER_VERTEXES)[..].try_into().unwrap();
    let us_data = DiamondBlue {
        inner_vertexes,
        query_result: None,
    };
    let xform = Transformation::new(x, y, theta);
    let shape = Shape::of_polygon(Box::new(BORDER_VERTEXES));
    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Blue,
        collision_damage: 8.0,
        hp: 10.0,
        engine_power: 10.0,
        tire_traction: 5.0,
    }).hitbox(xform, shape)
        .angular_power(1.0)
        .angular_traction(2.0)
        .act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .us_data(Box::new(us_data))
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<DiamondBlue>().unwrap();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    if let Some(qr) = us_data.query_result.take() {
        let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected qr={:?}", &*qr.borrow())};
        if let Some(cu) = cu_opt {
            let angle_to_player1 = f64::atan2(cu.y - xform.dy, cu.x - xform.dx);
            let angle_to_player2 = angle_to_player1 + std::f64::consts::PI;
            let _2_pi = 2.0 * std::f64::consts::PI;
            let angle_diffs = [angle_to_player1, angle_to_player2].map(|a2p| {
                let theta_diff = f64::abs((a2p - xform.dtheta) % _2_pi);
                f64::min(theta_diff, _2_pi - theta_diff)
            });

            let (x_xform, y_xform) = (f64::cos(xform.dtheta), f64::sin(xform.dtheta));
            let a2p = if angle_diffs[0] < angle_diffs[1] {
                angle_to_player1
            } else {
                angle_to_player2
            };
            let (x_a2p, y_a2p) = (f64::cos(a2p), f64::sin(a2p));
            let cp = cross_prod(x_xform, y_xform, x_a2p - x_xform, y_a2p - y_xform);
            let atheta = f64::signum(cp);
            ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate {atheta});
            let xlate_angle = if angle_diffs[0] < angle_diffs[1] {
                xform.dtheta
            } else {
                xform.dtheta + std::f64::consts::PI
            };
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate{ ax: f64::cos(xlate_angle), ay: f64::sin(xlate_angle) });
        }
    }
    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    let mut response = Act1Response::new();
    us_data.query_result = Some(response.add_query(query));
    response
}

fn cross_prod(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    x1 * y2 - x2 * y1
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<DiamondBlue>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let xlate = Vector::new(xform.dx as f32, xform.dy as f32);
    let border_outer_v = BORDER_VERTEXES.map(|p| p.rotated(xform.dtheta as f32)).map(|p| p + xlate);
    let border_inner_v = us_data.inner_vertexes.map(|p| p.rotated(xform.dtheta as f32)).map(|p| p + xlate);
    let border_dop = ctx.draw_ctx.do_thick_border(DrawContext::COLOR_NSU_BORDER, &border_outer_v, &border_inner_v);
    let inner_dop = ctx.draw_ctx.do_quad_fan(INNER_COLOR, border_inner_v);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT_FLYING, ctx.draw_ctx.dop_group(Box::new([border_dop, inner_dop])));
}