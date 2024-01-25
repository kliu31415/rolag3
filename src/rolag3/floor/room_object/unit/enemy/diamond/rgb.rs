use std::{rc::Rc, cell::RefCell, ops::Range};

use crate::{geometry::{shape::{Point, Shape, Vector}, util::get_inner_polygon}, rolag3::floor::{draw::{Color, DrawContext}, room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, Act1QueryResult, Act1QueryArgs}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, RofizObjType}, standard_unit_common::{RotateMove, TranslateMove}}, damage::DamageColor}, rofiz::rofiz_object::Transformation}};

/* DiamondRgb is a diamond composed of two equilateral-like triangles. It rotates towards the player and
   moves in the direction of the tips. It's spawned by BigCircleBlueDiamond. It comes in three different colors, each
   with different behavior:
   -Red: Alternates between short bursts of translating towards the player and rotating towards the player
   -Green: Like red, but with long bursts
   -Blue: Constantly translates and rotates towards the player
 */

pub const BORDER_VERTEXES: [Point; 4] = [Point::new(0.8, 0.0), Point::new(0.0, 0.5), Point::new(-0.8, 0.0), Point::new(0.0, -0.5)];

pub const INNER_COLOR_R: Color = Color::new(2.0, 0.03, 0.03, 1.0);
pub const INNER_COLOR_G: Color = Color::new(0.03, 1.3, 0.03, 1.0);
pub const INNER_COLOR_B: Color = Color::new(0.03, 0.03, 2.0, 1.0);

struct DiamondRgb {
    inner_vertexes: [Point; 4],
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    inner_color: Color,
    move_action: MoveAction,
    xlate_angle_override: Option<f64>,
}

enum XlateOrRotate {
    Xlate,
    Rotate,
}

enum MoveAction {
    AlwaysXlateAndRotate,
    AlternateXlateAndRotate {
        xlate_duration: Range<f64>,
        rotate_duration: Range<f64>,
        cur_action: XlateOrRotate,
        cur_action_end_age: f64,
    },
}

pub fn new_diamond_rgb(ctx: &mut NewRoomObjectContext, damage_color: DamageColor, x: f64, y: f64, theta: f64) -> StandardUnit1 {
    let inner_vertexes: [Point; 4] = get_inner_polygon(0.1, &BORDER_VERTEXES)[..].try_into().unwrap();
    let inner_color = match damage_color {
        DamageColor::Red => INNER_COLOR_R,
        DamageColor::Green => INNER_COLOR_G,
        DamageColor::Blue => INNER_COLOR_B,
        _ => panic!("unexpected damage_color={:?}", damage_color),
    };
    let move_action = match damage_color {
        DamageColor::Red => {
            MoveAction::AlternateXlateAndRotate { 
                xlate_duration: (1.0..1.0), 
                rotate_duration: (0.3..0.3), 
                cur_action: XlateOrRotate::Xlate,
                cur_action_end_age: 0.0, // on the first act1() call, this unit's move action will immediately switch to Rotate
            }
        }
        DamageColor::Green => {
            MoveAction::AlternateXlateAndRotate { 
                xlate_duration: (1.5..2.0), 
                rotate_duration: (0.5..1.0), 
                cur_action: XlateOrRotate::Xlate,
                cur_action_end_age: 0.0, // on the first act1() call, this unit's move action will immediately switch to Rotate
            }
        },
        DamageColor::Blue => MoveAction::AlwaysXlateAndRotate,
        _ => panic!("unexpected damage_color={:?}", damage_color),
    };
    let us_data = DiamondRgb {
        inner_color,
        inner_vertexes,
        query_result: None,
        move_action,
        xlate_angle_override: None,
    };
    let (engine_power, tire_traction, angular_power, angular_traction) = match damage_color {
        DamageColor::Red => (15.0, 20.0, 3.0, 10.0),
        DamageColor::Green => (25.0, 6.0, 3.0, 2.0),
        DamageColor::Blue => (10.0, 5.0, 1.0, 2.0),
        _ => panic!("unexpected damage_color={:?}", damage_color),
    };
    let xform = Transformation::new(x, y, theta);
    let shape = Shape::of_polygon(Box::new(BORDER_VERTEXES));
    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: 10.0,
        engine_power,
        tire_traction,
    }).hitbox(xform, shape)
        .angular_power(angular_power)
        .angular_traction(angular_traction)
        .act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .us_data(Box::new(us_data))
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<DiamondRgb>().unwrap();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());
    if let Some(qr) = us_data.query_result.take() {
        let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected qr={:?}", &*qr.borrow())};
        if let Some(cu) = cu_opt {
            let (xlate_enabled, rotate_enabled, keep_xlate_constant) = match &mut us_data.move_action {
                MoveAction::AlwaysXlateAndRotate => (true, true, false),
                MoveAction::AlternateXlateAndRotate { 
                    xlate_duration, 
                    rotate_duration, 
                    cur_action, 
                    cur_action_end_age,
                 } => {
                    let unit_age = ctx.su_ctx.su_common.get_unit_time();
                    if unit_age > *cur_action_end_age {
                        *cur_action = match cur_action {
                            XlateOrRotate::Xlate => XlateOrRotate::Rotate,
                            XlateOrRotate::Rotate => XlateOrRotate::Xlate,
                        };
                        let (start, end) = match cur_action {
                            XlateOrRotate::Xlate => (xlate_duration.start, xlate_duration.end),
                            XlateOrRotate::Rotate => (rotate_duration.start, rotate_duration.end),
                        };
                        *cur_action_end_age = unit_age + start + ctx.act1_ctx.get_randf64() * (end - start);
                    }
                    match cur_action {
                        XlateOrRotate::Xlate => (true, false, true),
                        XlateOrRotate::Rotate => (false, true, false),
                    }
                 }
            };
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
            // don't rotate if the angle is close enough. Otherwise the unit might "vibrate"" around 0.0 due to atheta
            // rapidly changing signs
            if f64::hypot(x_a2p - x_xform, y_a2p - y_xform) > 0.01 {
                let cp = cross_prod(x_xform, y_xform, x_a2p - x_xform, y_a2p - y_xform);
                let atheta = f64::signum(cp);
                if rotate_enabled {
                    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate {atheta});
                } else {
                    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Decelerate);
                }
            } else {
                ctx.su_ctx.su_common.set_rotate_move(RotateMove::Decelerate);
            }

            let xlate_angle = if keep_xlate_constant && us_data.xlate_angle_override.is_some() {
                us_data.xlate_angle_override.unwrap()
            } else if angle_diffs[0] < angle_diffs[1] {
                xform.dtheta
            } else {
                xform.dtheta + std::f64::consts::PI
            };

            if keep_xlate_constant {
                us_data.xlate_angle_override = Some(xlate_angle);
            } else {
                us_data.xlate_angle_override = None;
            }

            if xlate_enabled {
                let movement = TranslateMove::Accelerate{ ax: f64::cos(xlate_angle), ay: f64::sin(xlate_angle) };
                ctx.su_ctx.su_common.set_translate_move(movement);
            } else {
                ctx.su_ctx.su_common.set_translate_move(TranslateMove::Decelerate);
            }
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
    let us_data = ctx.su_ctx.us_data.downcast_mut::<DiamondRgb>().unwrap();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let xlate = Vector::new(xform.dx as f32, xform.dy as f32);
    let border_outer_v = BORDER_VERTEXES.map(|p| p.rotated(xform.dtheta as f32)).map(|p| p + xlate);
    let border_inner_v = us_data.inner_vertexes.map(|p| p.rotated(xform.dtheta as f32)).map(|p| p + xlate);
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_outer_v, &border_inner_v);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(us_data.inner_color);
    let inner_dop = ctx.draw_ctx.do_quad_fan(inner_color, border_inner_v);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT_FLYING, ctx.draw_ctx.dop_group(Box::new([border_dop, inner_dop])));
}