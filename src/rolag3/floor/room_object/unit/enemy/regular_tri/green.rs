use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1}, standard_unit_common::{TranslateMove, Budeb, BudebMaxSpeed, BudebExpiry}}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point}, util::{regular_polygon, get_inner_polygon, rotate_polygon}}};

/* RegtriGreen is a green triangle that continuously moves in the direction of the player.
   It slows down when it takes damage. The slow factor is represented by "excitement".
   Note that higher excitement results in a bigger slowdown (rather than a speedup).
*/

const INNER_COLOR: Color = Color::new(0.05, 0.8, 0.05, 1.0);

pub struct RegtriGreen {
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    border_vertexes: [Point; 3],
    inner_vertexes: [Point; 3],
    hp_last_tick: Option<f64>,
    excitement: f64,
}

pub fn new_regtri_green(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let mut border_vertexes: [Point; 3] = regular_polygon(3, 1.4)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_6, &mut border_vertexes);
    let inner_vertexes: [Point; 3] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = RegtriGreen { 
        query_result: None,
        border_vertexes,
        inner_vertexes,
        hp_last_tick: None,
        excitement: 0.0,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Green,
        hp: 20.0,
        engine_power: 13.0,
        tire_traction: 25.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RegtriGreen>().unwrap();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
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

    match us_data.hp_last_tick {
        Some(prev) => {
            let cur = ctx.su_ctx.su_common.get_cur_hp();
            us_data.excitement *= f64::powf(0.1, tick_len);
            us_data.excitement += f64::max(0.0, prev - cur);
            us_data.hp_last_tick = Some(cur);
        }
        None => us_data.hp_last_tick = Some(ctx.su_ctx.su_common.get_cur_hp()),
    }
    assert!(us_data.excitement >= 0.0);
    let speed_mult = f64::recip(f64::cbrt(1.0 + us_data.excitement));
    ctx.su_ctx.su_common.apply_budeb(&Budeb::SpeedMult(BudebMaxSpeed::new(speed_mult, BudebExpiry::OneTick)));

    // x and y in the query shouldn't matter because there's usually one player. I set them anyway in case there are
    // multiple players in the future
    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    let mut response = Act1Response::new();
    us_data.query_result = Some(response.add_query(query));
    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RegtriGreen>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), DrawContext::COLOR_NSU_BORDER);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), INNER_COLOR);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let border_vertexes = us_data.border_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let inner_vertexes = us_data.inner_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &inner_vertexes);
    let inner_dop = ctx.draw_ctx.do_tri(inner_color, inner_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, inner_dop])));
}