use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1}, standard_unit_common::{TranslateMove, RotateMove}}, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point}, util::get_inner_polygon, star::get_star_shape}};

/* Thinstar5Green is stationary at first, but starts moving when it takes damage.
   It rapidly moves in the directly of the player. It releases a wave of 32 projectiles upon death.
*/

const RADIUS: f64 = 1.1;
const MAX_HP: f64 = 20.0;
const BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const INNER_COLOR_INACTIVE: Color = Color::new(0.02, 0.2, 0.02, 1.0);
const INNER_COLOR_ACTIVE: Color = Color::new(0.02, 1.5, 0.02, 1.0);
const PROJ_COLOR: Color = Color::new(0.02, 1.5, 0.02, 1.0);

pub struct ThinStar5Green {
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    border_vertexes: [Point; 10],
    inner_vertexes: [Point; 10],
}

pub fn new_thinstar5_green(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let border_vertexes: [Point; 10] = get_star_shape(5, 0.4, RADIUS as f32, -0.1 * std::f32::consts::PI)[..].try_into().unwrap();
    let inner_vertexes: [Point; 10] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = ThinStar5Green { 
        query_result: None,
        border_vertexes,
        inner_vertexes,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Green,
        collision_damage: 10.0,
        hp: MAX_HP,
        engine_power: 15.0,
        tire_traction: 10.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .angular_power(2.0)
        .angular_traction(20.0)
        .remove_immediately_on_death(false)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ThinStar5Green>().unwrap();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    if *ctx.su_ctx.is_dead {
        let mut response = Act1Response::new().remove_room_obj(ctx.su_ctx.md.get_ref());
        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
        let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
        let num_proj = 32;
        for i in 0..num_proj {
            let angle = xform.dtheta + i as f64 * std::f64::consts::FRAC_PI_2 / (num_proj as f64);
            let proj_speed = 10.0;
            // Use a point in the middle of the projectile as the center. We want the center to be the middle of
            // a projectile because many things, e.g. black hole force. assume the center is around the middle.
            let proj = Projectile2Builder::new(
            Projectile2BuilderReq {
                team: Team::Enemy,
                damage_color: DamageColor::Green,
                damage: 3.0,
                owner: self_as_weak.clone(),
                velocity_x: proj_speed * f64::cos(angle),
                velocity_y: proj_speed * f64::sin(angle),
                xform,
                shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: 0.2  },
                color: PROJ_COLOR,
            }
            ).build(&mut nfo_ctx);
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
        return response;
    }

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

    let mut response = Act1Response::new();
    if ctx.su_ctx.su_common.get_cur_hp() < MAX_HP {
        ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate { atheta: -1.0 });
        let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
        us_data.query_result = Some(response.add_query(query));
    }
    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ThinStar5Green>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), BORDER_COLOR);
    let inner_color = if ctx.su_ctx.su_common.get_cur_hp() < MAX_HP {
        INNER_COLOR_ACTIVE
    } else {
        INNER_COLOR_INACTIVE
    };
    let inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), inner_color);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let border_vertexes = us_data.border_vertexes
        .map(|v| v.rotated(xform.dtheta as f32))
        .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let inner_vertexes = us_data.inner_vertexes
        .map(|v| v.rotated(xform.dtheta as f32))
        .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &inner_vertexes);
    let center = Point::new(xform.dx as f32, xform.dy as f32);
    let tri_fan_vertexes = [center].into_iter()
        .chain(inner_vertexes.iter().cloned())
        .chain(inner_vertexes[..1].iter().cloned())
        .collect::<Box<_>>();
    let inner_dop = ctx.draw_ctx.do_tri_fan(inner_color, &tri_fan_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, inner_dop])));
}