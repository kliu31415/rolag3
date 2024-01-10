use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1}, standard_unit_common::{TranslateMove, RotateMove}}, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point}, util::{get_inner_polygon, rotate_polygon}, star::get_star_shape}};

/* Fatstar4Green slowly moves in the directly of the player. It releases a wave of 4 projectiles upon death.
*/

pub const RADIUS: f64 = 1.0;
const BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const INNER_COLOR: Color = Color::new(0.05, 0.8, 0.05, 1.0);
const PROJ_COLOR: Color = Color::new(0.05, 0.8, 0.05, 1.0);

pub struct FatStar4Green {
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
    border_vertexes: [Point; 8],
    inner_vertexes: [Point; 8],
}

pub fn new_fatstar4_green(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let mut border_vertexes: [Point; 8] = get_star_shape(4, 0.5, RADIUS as f32, std::f32::consts::FRAC_PI_4)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let inner_vertexes: [Point; 8] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = FatStar4Green { 
        query_result: None,
        border_vertexes,
        inner_vertexes,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Green,
        collision_damage: 10.0,
        hp: 20.0,
        engine_power: 5.0,
        tire_traction: 5.0,
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
    let us_data = ctx.su_ctx.us_data.downcast_mut::<FatStar4Green>().unwrap();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    if *ctx.su_ctx.is_dead {
        let mut response = Act1Response::new().remove_room_obj(ctx.su_ctx.md.get_ref());
        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
        let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
        for i in 0..4 {
            let angle = xform.dtheta + i as f64 * std::f64::consts::FRAC_PI_2;
            let proj_speed = 13.0;
            // TODO: the projectile visually looks like a diamond. However, I intended for it to be a triangle.
            // Why is it a diamond?
            let vertex1 = us_data.inner_vertexes[i*2+1];
            let vertex2 = us_data.inner_vertexes[(i*2+2) % 8];
            let vertex3 = us_data.inner_vertexes[(i*2+3) % 8];
            let proj = Projectile2Builder::new(
            Projectile2BuilderReq {
                team: Team::Enemy,
                damage_color: DamageColor::Green,
                damage: 4.0,
                owner: self_as_weak.clone(),
                lifespan: 8.0,
                velocity_x: proj_speed * f64::cos(angle),
                velocity_y: proj_speed * f64::sin(angle),
                xform,
                shape: Proj2Shape::TriFan { center: vertex1, vertexes: Box::new([vertex1, vertex2, vertex3]) },
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
    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate { atheta: 1.0 });

    // x and y in the query shouldn't matter because there's usually one player. I set them anyway in case there are
    // multiple players in the future
    let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
    let mut response = Act1Response::new();
    us_data.query_result = Some(response.add_query(query));
    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<FatStar4Green>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), BORDER_COLOR);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), INNER_COLOR);
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