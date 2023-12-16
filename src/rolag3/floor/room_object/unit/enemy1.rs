use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, HandleCollisionResponse, Team}, projectile::projectile2::NewProjectile2Args, damage::DamageColor}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::shape::{Shape, Point}};

use super::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, SuHandleCollisionContext, HandleCollisionLogic}, standard_unit_common::TranslateMove};

use std::{f64::consts::PI, cell::RefCell, rc::Rc};

const SIDE_LEN: f32 = 1.2;

pub struct Enemy1 {
    accel_xy_angle: f64,
    spit_projectile_start: Option<SpitProjectileInfo>,
    should_reset_velocity: bool,
}

struct SpitProjectileInfo {
    start: f64,
    proj_spit: bool,
    proj_dx: f64,
    proj_dy: f64,
}

pub fn new_enemy1(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_square(-SIDE_LEN/2.0, -SIDE_LEN/2.0, SIDE_LEN);
    let us_data = Enemy1 {
        accel_xy_angle: 2.0 * PI * ctx.get_randf64(),
        spit_projectile_start: None,
        should_reset_velocity: false,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Blue,
        hp: 30.0,
        engine_power: 40.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Enemy1>().unwrap();

    let mut response = Act1Response::new();
    let tick_len = ctx.act1_ctx.get_tick_length();

    if us_data.spit_projectile_start.is_none() && ctx.act1_ctx.get_randf64() < tick_len {
        if let Some(player_xy) = ctx.act1_ctx.get_team_closest_location(Team::Player) {
            let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
            let proj_velocity = 25.0;
            let theta = f64::atan2(player_xy.y - xform.dy, player_xy.x - xform.dx);
            let proj_dx = proj_velocity * f64::cos(theta);
            let proj_dy = proj_velocity * f64::sin(theta);
            us_data.spit_projectile_start = Some(SpitProjectileInfo { start: ctx.act1_ctx.get_room_time(), proj_spit: false, proj_dx, proj_dy });
        }
    }

    if let Some(ref mut sps) = us_data.spit_projectile_start {
        if !sps.proj_spit && ctx.act1_ctx.get_room_time() - sps.start > 0.5 {
            sps.proj_spit = true;
            let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
            let self_as_weak = ctx.act1_ctx.get_self_as_weak();
            let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
            let proj = NewProjectile2Args{
                team: Team::Enemy,
                damage_color: DamageColor::Blue,
                owner: self_as_weak,
                lifespan: 2.0,
                velocity_x: sps.proj_dx,
                velocity_y: sps.proj_dy,
                xform,
                center: Point::new(0.0, 0.0), 
                vertexes: vec![Point::new(-0.4, -0.4), Point::new(0.4, -0.4), Point::new(0.4, 0.4), Point::new(-0.4, 0.4)].into_boxed_slice(), 
                color: Color::new(0.0, 0.0, 15.0, 1.0),
            }.new(&mut nfo_ctx);
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
        if ctx.act1_ctx.get_room_time() - sps.start > 1.0 {
            us_data.spit_projectile_start = None;
        }
    }

    if us_data.should_reset_velocity {
        ctx.su_ctx.su_common.set_translate_move(TranslateMove::ResetVelocity);
        us_data.accel_xy_angle = 2.0 * PI * ctx.act1_ctx.get_randf64();
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
    let color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(0.1, 0.1, 1.0, 1.0));
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let x = xform.dx as f32 - SIDE_LEN / 2.0;
    let y = xform.dy as f32 - SIDE_LEN / 2.0;
    let w = SIDE_LEN;
    let h = SIDE_LEN;
    let vertexes = [
        Point::new(x, y),
        Point::new(x + w, y),
        Point::new(x + w, y + h),
        Point::new(x, y + h),
    ];
    let dop1 = ctx.draw_ctx.do_quad(color, vertexes);
    let eye_border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(0.0, 0.0, 0.0, 1.0));
    let eye_sclera_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(1.0, 1.0, 1.0, 1.0));
    let eye_iris_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(0.0, 0.0, 0.0, 1.0));
    let dop2 = ctx.draw_ctx.do_eye(Point::new((xform.dx - 0.25) as f32, (xform.dy - 0.25) as f32), Point::new((xform.dx - 0.25) as f32, (xform.dy - 0.25) as f32), 0.4, 0.25, 0.1, 0.05, eye_border_color, eye_sclera_color, eye_iris_color);
    let dop3 = ctx.draw_ctx.do_eye(Point::new((xform.dx + 0.25) as f32, (xform.dy - 0.25) as f32), Point::new((xform.dx + 0.25) as f32, (xform.dy - 0.25) as f32), 0.4, 0.25, 0.1, 0.05, eye_border_color, eye_sclera_color, eye_iris_color);
    let mouth_border_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(0.0, 0.0, 0.0, 1.0));
    let mouth_inner_color = ctx.su_ctx.su_common.get_draw_color(ctx.draw_ctx.get_room_time(), Color::new(0.5, 0.5, 0.5, 1.0));
    let dop4 = ctx.draw_ctx.do_mouth_smile(((1.0 + f64::sin(3.0 * ctx.draw_ctx.get_room_time())) / 2.0) as f32, Point::new(xform.dx as f32, (xform.dy + 0.25) as f32), 0.6, 0.29, 0.05, mouth_border_color, mouth_inner_color);
    let dop_group = ctx.draw_ctx.dop_group(vec![dop1, dop2, dop3, dop4].into_boxed_slice());
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop_group);
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Enemy1>().unwrap();
    if !ctx.hc_ctx.get_other().borrow().is_spectral() {
        us_data.should_reset_velocity = true;
    }
    HandleCollisionResponse::new()
}