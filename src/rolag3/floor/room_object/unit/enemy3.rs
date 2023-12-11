use crate::{rolag3::floor::{room_object::room_object_def::{NewRoomObjectContext, Team, Act1Response}, rofiz::rofiz_object::Transformation, draw::{Color, FloorDrawCoordinate, DrawContext}}, geometry::shape::Shape};

use super::standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, StandardUnit1, SuAct1Context, SuDrawContext};

const SIDE_LEN: f32 = 1.8;

pub fn new_enemy3(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_square(-SIDE_LEN/2.0, -SIDE_LEN/2.0, SIDE_LEN);
    let us_data = Enemy3Data {
        move_charge: None,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        hp: 30.0,
        engine_power: 40.0,
        tire_traction: 10.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

pub struct Enemy3Data {
    move_charge: Option<MoveCharge>
}

struct MoveCharge {
    started_at: f64,
    velocity_x: f64,
    velocity_y: f64,
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.us_data.downcast_mut::<Enemy3Data>().unwrap();
    let tick_len = ctx.act1_ctx.get_tick_length();

    match us_data.move_charge {
        Some(ref mc) => {
            let since_start = ctx.act1_ctx.get_room_time() - mc.started_at;
            if since_start > 1.0 {
                ctx.su_ctx.su_common.accelerate_ro_xy(tick_len, mc.velocity_x, mc.velocity_y);
                if since_start > 4.0 {
                    us_data.move_charge = None;
                }
            }
        },
        None => {
            ctx.su_ctx.su_common.decelerate_ro_xy(tick_len);
            if us_data.move_charge.is_none() && ctx.act1_ctx.get_randf64() < tick_len {
                if let Some(player_xy) = ctx.act1_ctx.get_team_closest_location(Team::Player) {
                    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
                    let theta = f64::atan2(player_xy.y - xform.dy, player_xy.x - xform.dx);
                    let velocity_x = f64::cos(theta);
                    let velocity_y = f64::sin(theta);
                    us_data.move_charge = Some(MoveCharge { started_at: ctx.act1_ctx.get_room_time(), velocity_x, velocity_y});
                }
            }
        },
    }

    Act1Response::new()
}

fn draw(ctx: &mut SuDrawContext) {
    let color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(2.0, 0.0, 0.0, 1.0));
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