use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{draw::{Color, DrawContext}, room_object::{room_object_def::{Act1QueryResult, NewRoomObjectContext, Team, Act1Response, Act1QueryArgs, HandleCollisionResponse}, unit::{standard_unit1::{StandardUnit1, StandardUnit1BuilderReq, StandardUnit1Builder, HandleCollisionLogic, SuAct1Context, SuDrawContext, SuHandleCollisionContext}, standard_unit_common::TranslateMove}, damage::DamageColor, projectile::projectile2::{Proj2Shape, Projectile2BuilderReq, Projectile2Builder}}, rofiz::rofiz_object::Transformation}, geometry::{shape::{Shape, Point}, util::{rotate_polygon, regular_polygon, get_inner_polygon}}};

/* SquareRgbCircle randomly translates. It occasionally spits a projectile in the player's direction.
*/

const PROJ_RADIUS: f32 = 0.3;

pub struct SquareRgbCircle {
    border_vertexes: [Point; 4],
    outer_vertexes: [Point; 4],
    outer_color: Color,
    inner_color: Color,
    proj_color: Color,
    accel_xy_angle: f64,
    spit_projectile_start: Option<SpitProjectileInfo>,
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
}

struct SpitProjectileInfo {
    start: f64,
    proj_spit: bool,
    num_proj: usize,
    angle_between_proj: f64,
    proj_speed: f64,
    proj_angle: f64,
}

pub fn new_square_rgb_circle(ctx: &mut NewRoomObjectContext, damage_color: DamageColor, x: f64, y: f64) -> StandardUnit1 {
    let mut border_vertexes: [Point; 4] = regular_polygon(4, 1.3)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let outer_vertexes: [Point; 4] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(outer_vertexes));

    let outer_color = match damage_color {
        DamageColor::Red => Color::new(0.5, 0.0, 0.0, 1.0),
        DamageColor::Green => Color::new(0.0, 0.4, 0.0, 1.0),
        DamageColor::Blue => Color::new(0.0, 0.0, 0.5, 1.0),
        _ => panic!("unexpected damage color {:?}", damage_color),
    };

    let inner_color = match damage_color {
        DamageColor::Red => Color::new(1.0, 0.0, 0.0, 1.0),
        DamageColor::Green => Color::new(0.0, 0.8, 0.0, 1.0),
        DamageColor::Blue => Color::new(0.0, 0.0, 1.0, 1.0),
        _ => panic!("unexpected damage color {:?}", damage_color),
    };

    let proj_color = match damage_color {
        DamageColor::Red => Color::new(5.0, 0.01, 0.01, 1.0),
        DamageColor::Green => Color::new(0.01, 1.5, 0.01, 1.0),
        DamageColor::Blue => Color::new(0.06, 0.06, 14.0, 1.0),
        _ => panic!("unexpected damage color {:?}", damage_color),
    };

    let us_data = SquareRgbCircle {
        border_vertexes,
        outer_vertexes,
        outer_color,
        inner_color,
        proj_color,
        accel_xy_angle: 2.0 * std::f64::consts::PI * ctx.get_randf64(),
        spit_projectile_start: None,
        query_result: None,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: 15.0,
        engine_power: 15.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgbCircle>().unwrap();

    let mut response = Act1Response::new();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
    let time = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());

    if let Some(ref qr) = us_data.query_result {
        match &*qr.borrow() {
            Act1QueryResult::ClosestUnit(v) => {
                if let Some(closest) = v {
                    let proj_angle = f64::atan2(closest.y - xform.dy, closest.x - xform.dx);
                    let num_proj = ctx.su_ctx.damage_color.to_rgb_idx() + 1;
                    us_data.spit_projectile_start = Some(SpitProjectileInfo { 
                        start: time, 
                        proj_spit: false,
                        num_proj,
                        angle_between_proj: std::f64::consts::FRAC_PI_8,
                        proj_speed: 12.0,
                        proj_angle,
                    });
                }
            }
            _ => panic!("unexpected Act1QueryResult. Expected ClosestUnit, got {:?}", qr),
        }
        us_data.query_result = None;
    }

    if us_data.spit_projectile_start.is_none() && ctx.act1_ctx.get_randf64() < tick_len {
        // x and y in the query shouldn't matter because there's usually one player. I set them anyway in case there are
        // multiple players in the future
        let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
        us_data.query_result = Some(response.add_query(query));
    }

    if let Some(ref mut sps) = us_data.spit_projectile_start {
        if !sps.proj_spit && time - sps.start > 0.5 {
            sps.proj_spit = true;
            let self_as_weak = ctx.act1_ctx.get_self_as_weak();
            let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
            let shape = Proj2Shape::Circle {
                x: 0.0,
                y: 0.0,
                r: PROJ_RADIUS,
            };
            for i in 0..sps.num_proj {
                let angle = sps.proj_angle + sps.angle_between_proj * (i as f64 - 0.5 * (sps.num_proj - 1) as f64);
                let proj = Projectile2Builder::new(
                    Projectile2BuilderReq{
                        team: Team::Enemy,
                        damage_color: *ctx.su_ctx.damage_color,
                        damage: 3.0,
                        owner: self_as_weak.clone(),
                        velocity_x: sps.proj_speed * f64::cos(angle),
                        velocity_y: sps.proj_speed * f64::sin(angle),
                        xform,
                        shape: shape.clone(),
                        color: us_data.proj_color,
                    }
                ).build(&mut nfo_ctx);
                response.add_room_obj(Rc::new(RefCell::new(proj)));
            }
        }
        if time - sps.start > 1.0 {
            us_data.spit_projectile_start = None;
        }
    }

    match us_data.spit_projectile_start {
        Some(_) => {
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Decelerate);
        }
        None => {
            us_data.accel_xy_angle += 10.0 * f64::sqrt(tick_len) * (ctx.act1_ctx.get_randf64() - 0.5);
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: f64::cos(us_data.accel_xy_angle), ay: f64::sin(us_data.accel_xy_angle)});
        }
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgbCircle>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(us_data.outer_color);
    let inner_color = match us_data.spit_projectile_start {
        Some(ref x) => {
            let diff = ctx.su_ctx.su_common.get_unit_time() - x.start;
            assert!(diff>=0.0 && diff<=1.0);
            let lerp_a = 2.0 * (1.0 - f64::abs(0.5 - diff));
            Color::lerp(us_data.inner_color, us_data.proj_color, lerp_a as f32)
        },
        None => us_data.inner_color,
    };
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let border_vertexes = us_data.border_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let outer_vertexes = us_data.outer_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let outer_dop = ctx.draw_ctx.do_quad_fan(outer_color, outer_vertexes);
    let inner_dop = ctx.draw_ctx.do_circle(inner_color, Point::new(xform.dx as f32, xform.dy as f32), PROJ_RADIUS);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, outer_dop, inner_dop])));
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgbCircle>().unwrap();
    if !ctx.hc_ctx.is_other_spectral() {
        us_data.accel_xy_angle = 2.0 * std::f64::consts::PI * ctx.hc_ctx.get_randf64();
    }
    HandleCollisionResponse::new()
}