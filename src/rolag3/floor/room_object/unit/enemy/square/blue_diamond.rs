use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{draw::{Color, DrawContext}, room_object::{room_object_def::{Act1QueryResult, NewRoomObjectContext, Team, Act1Response, Act1QueryArgs, HandleCollisionResponse}, unit::{standard_unit1::{StandardUnit1, StandardUnit1BuilderReq, StandardUnit1Builder, HandleCollisionLogic, SuAct1Context, SuDrawContext, SuHandleCollisionContext}, standard_unit_common::TranslateMove}, damage::DamageColor, projectile::projectile2::{Proj2Shape, NewProjectile2Args}}, rofiz::rofiz_object::Transformation}, geometry::{shape::{Shape, Point}, util::{rotate_polygon, regular_polygon, get_inner_polygon}}};

/* SquareBlueDiamond randomly translates. Enemy1 occasionally spits a projectile in the player's direction.
*/

const BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const OUTER_COLOR: Color = Color::new(0.0, 0.0, 1.0, 1.0);
const INNER_COLOR: Color = Color::new(0.1, 0.1, 2.0, 1.0);
const PROJ_COLOR: Color = Color::new(0.2, 0.2, 13.0, 1.0);

pub struct SquareBlueDiamond {
    border_vertexes: [Point; 4],
    outer_vertexes: [Point; 4],
    proj_vertexes: [Point; 4],
    accel_xy_angle: f64,
    spit_projectile_start: Option<SpitProjectileInfo>,
    should_reset_velocity: bool,
    query_result: Option<Rc<RefCell<Act1QueryResult>>>,
}

struct SpitProjectileInfo {
    start: f64,
    proj_spit: bool,
    proj_dx: f64,
    proj_dy: f64,
}

pub fn new_square_blue_circle(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let mut border_vertexes: [Point; 4] = regular_polygon(4, 1.0)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let outer_vertexes: [Point; 4] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let proj_vertexes: [Point; 4] = regular_polygon(4, 0.35)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(outer_vertexes));
    let us_data = SquareBlueDiamond {
        border_vertexes,
        outer_vertexes,
        proj_vertexes,
        accel_xy_angle: 2.0 * std::f64::consts::PI * ctx.get_randf64(),
        spit_projectile_start: None,
        should_reset_velocity: false,
        query_result: None,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Blue,
        hp: 30.0,
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
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareBlueDiamond>().unwrap();

    let mut response = Act1Response::new();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
    let time = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());

    if let Some(ref qr) = us_data.query_result {
        match &*qr.borrow() {
            Act1QueryResult::ClosestUnit(v) => {
                if let Some(closest) = v {
                    let proj_velocity = 12.0;
                    let theta = f64::atan2(closest.y - xform.dy, closest.x - xform.dx);
                    let proj_dx = proj_velocity * f64::cos(theta);
                    let proj_dy = proj_velocity * f64::sin(theta);
                    us_data.spit_projectile_start = Some(SpitProjectileInfo { start: time, proj_spit: false, proj_dx, proj_dy });
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
            let shape = Proj2Shape::TriFan {
                center: Point::new(0.0, 0.0), 
                vertexes: Box::new(us_data.proj_vertexes), 
            };
            let proj = NewProjectile2Args{
                team: Team::Enemy,
                damage_color: DamageColor::Blue,
                damage: 3.0,
                owner: self_as_weak,
                lifespan: 5.0,
                velocity_x: sps.proj_dx,
                velocity_y: sps.proj_dy,
                xform,
                shape,
                color: PROJ_COLOR,
            }.new(&mut nfo_ctx);
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
        if time - sps.start > 1.0 {
            us_data.spit_projectile_start = None;
        }
    }

    if us_data.should_reset_velocity {
        ctx.su_ctx.su_common.set_translate_move(TranslateMove::ResetVelocity);
        us_data.accel_xy_angle = 2.0 * std::f64::consts::PI * ctx.act1_ctx.get_randf64();
        us_data.should_reset_velocity = false;
    } else {
        match us_data.spit_projectile_start {
            Some(_) => {
                ctx.su_ctx.su_common.set_translate_move(TranslateMove::ResetVelocity);
            }
            None => {
                us_data.accel_xy_angle += 10.0 * f64::sqrt(tick_len) * (ctx.act1_ctx.get_randf64() - 0.5);
                ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: f64::cos(us_data.accel_xy_angle), ay: f64::sin(us_data.accel_xy_angle)});
            }
        }
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareBlueDiamond>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), BORDER_COLOR);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), OUTER_COLOR);
    let inner_color = match us_data.spit_projectile_start {
        Some(ref x) => {
            let diff = ctx.su_ctx.su_common.get_unit_time() - x.start;
            assert!(diff>=0.0 && diff<=1.0);
            let lerp_a = f64::powi(2.0 * (1.0 - f64::abs(0.5 - diff)), 2);
            Color::lerp(INNER_COLOR, PROJ_COLOR, lerp_a as f32)
        },
        None => INNER_COLOR,
    };
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let border_vertexes = us_data.border_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let outer_vertexes = us_data.outer_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let inner_vertexes = us_data.proj_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_tri_fan_border(border_color, &border_vertexes, &outer_vertexes);
    let outer_dop = ctx.draw_ctx.do_quad_fan(outer_color, outer_vertexes);
    let inner_dop = ctx.draw_ctx.do_quad_fan(inner_color, inner_vertexes);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, outer_dop, inner_dop])));
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareBlueDiamond>().unwrap();
    if !ctx.hc_ctx.get_other().borrow().is_spectral() {
        us_data.should_reset_velocity = true;
    }
    HandleCollisionResponse::new()
}