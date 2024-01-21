use std::{cell::RefCell, rc::{Weak, Rc}};

use crate::{rolag3::floor::{room_object::{unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext}, enemy::diamond::{self, rgb::new_diamond_rgb}}, room_object_def::{NewRoomObjectContext, Act1Response, Team, RoomObject}, damage::DamageColor}, draw::{Color, DrawContext}, rofiz::rofiz_object::Transformation}, geometry::shape::{Point, Vector, Shape}};

/* BigCircleRgbDiamond is a stationary unit that doesn't attack directly. It periodically spawns children DiamondRgbs,
   up to 5 at a time.
 */

const OUTER_COLOR_R: Color = Color::new(0.3, 0.01, 0.01, 1.0);
const INNER_COLOR_R: Color = Color::new(1.0, 0.01, 0.01, 1.0);

const OUTER_COLOR_G: Color = Color::new(0.01, 0.3, 0.01, 1.0);
const INNER_COLOR_G: Color = Color::new(0.01, 0.8, 0.01, 1.0);

const OUTER_COLOR_B: Color = Color::new(0.01, 0.01, 0.3, 1.0);
const INNER_COLOR_B: Color = Color::new(0.01, 0.01, 1.0, 1.0);

const BORDER_RADIUS: f32 = 1.5;
const OUTER_RADIUS: f32 = 1.4;

struct BigCircleBlueDiamond {
    since_last_birth: f64,
    children: Vec<Weak<RefCell<dyn RoomObject>>>,
    birth_action: Option<BirthAction>,
    outer_color: Color,
    inner_color_lerp1: Color,
    inner_color_lerp2: Color,
}

struct BirthAction {
    end_time: f64,
}

pub fn new_big_circle_rgb_diamond(ctx: &mut NewRoomObjectContext, damage_color: DamageColor, x: f64, y: f64) -> StandardUnit1 {
    let (outer_color, inner_color_lerp1, inner_color_lerp2) = match damage_color {
        DamageColor::Red => (OUTER_COLOR_R, diamond::rgb::INNER_COLOR_R, INNER_COLOR_R),
        DamageColor::Green => (OUTER_COLOR_G, diamond::rgb::INNER_COLOR_G, INNER_COLOR_G),
        DamageColor::Blue => (OUTER_COLOR_B, diamond::rgb::INNER_COLOR_B, INNER_COLOR_B),
        _ => panic!("unexpected damage_color={:?}", damage_color),
    };
    let us_data = BigCircleBlueDiamond {
        since_last_birth: 0.0,
        children: Vec::new(),
        birth_action: None,
        outer_color,
        inner_color_lerp1,
        inner_color_lerp2,
    };
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), BORDER_RADIUS);
    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: 20.0,
        engine_power: 0.0,
        tire_traction: 0.0,
    }).hitbox(xform, shape)
        .act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<BigCircleBlueDiamond>().unwrap();
    let mut response = Act1Response::new();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    us_data.children.retain(|x| x.strong_count() > 0);
    match us_data.birth_action {
        Some(ref action) => {
            if unit_age > action.end_time {
                let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
                let nro_ctx = &mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
                let child = Rc::new(RefCell::new(new_diamond_rgb(nro_ctx, *ctx.su_ctx.damage_color, xform.dx, xform.dy, std::f64::consts::FRAC_PI_2))) as _;
                us_data.children.push(Rc::downgrade(&child));
                response.add_room_obj(child);
                us_data.since_last_birth = 0.0;
                us_data.birth_action = None;
            }
        }
        None => {
            us_data.since_last_birth += ctx.su_ctx.su_common.get_unit_tick_len();
            if us_data.since_last_birth > 1.5 
            && ctx.act1_ctx.get_randf64() < ctx.su_ctx.su_common.get_unit_tick_len()
            && us_data.children.len() < 5 {
             us_data.birth_action = Some(BirthAction {
                end_time: unit_age + 0.8,
            });
         }
        }
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<BigCircleBlueDiamond>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let inner_color = ctx.su_ctx.su_common.get_draw_color(us_data.outer_color);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let dop_circle = ctx.draw_ctx.do_concentric_circle(
        inner_color, 
        outer_color, 
        Point::new(xform.dx as f32, xform.dy as f32), 
        OUTER_RADIUS, 
        BORDER_RADIUS,
    );
    let xlate = Vector::new(xform.dx as f32, xform.dy as f32);
    let inner_vertexes = diamond::rgb::BORDER_VERTEXES.map(|p| p.rotated(std::f32::consts::FRAC_PI_2)).map(|p| p + xlate);
    let lerp_t = f64::min(1.0, 3.0 * match us_data.birth_action {
        Some(ref action) => {
            action.end_time - unit_age
        }
        None => {
            us_data.since_last_birth
        }
    }) as f32;
    let inner_color = Color::lerp(us_data.inner_color_lerp1, us_data.inner_color_lerp2, lerp_t);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(inner_color);
    let inner_dop = ctx.draw_ctx.do_quad_fan(inner_color, inner_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([dop_circle, inner_dop])));
}