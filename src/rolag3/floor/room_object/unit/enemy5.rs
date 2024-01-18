use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, HandleCollisionResponse, Team}, projectile::projectile2::{Projectile2BuilderReq, Proj2Shape, Projectile2Builder}, damage::DamageColor}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Polygon, Point}, util::regular_polygon}};

use super::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, SuHandleCollisionContext, HandleCollisionLogic}, standard_unit_common::TranslateMove};

use std::{cell::RefCell, rc::Rc};

/* Enemy5 is a red square that randomly translates in cardinal directions and periodically shoots a 
   wave of 4 projectiles in the cardinal directions. It changes directions upon colliding with a nonspectral object.
*/

const OUTER_COLOR: Color = Color::new(2.0, 0.0, 0.0, 1.0);
const INNER_COLOR: Color = Color::new(3.0, 0.0, 0.0, 1.0);
const PROJ_COLOR: Color = Color::new(7.0, 0.3, 0.3, 1.0);

pub struct Enemy5 {
    spit_projectile_start: Option<SpitProjectileInfo>,
    border: Polygon,
    outer: Polygon,
    inner: Polygon,
    translate_dir: i64,
    should_reset_velocity: bool,
}

struct SpitProjectileInfo {
    start: f64,
    proj_spit: bool,
}

pub fn new_enemy5(ctx: &mut NewRoomObjectContext, x: f64, y: f64) -> StandardUnit1 {
    let xform = Transformation::new(x, y, std::f64::consts::FRAC_PI_4);
    let border = Polygon::new(regular_polygon(4, 2.0));
    let outer = Polygon::new(regular_polygon(4, 1.8));
    let inner = Polygon::new(regular_polygon(4, 0.8));
    let shape = Shape::Polygon(border.clone());
    let us_data = Enemy5 {
        spit_projectile_start: None,
        border,
        outer,
        inner,
        translate_dir: ctx.get_randi64(0..4),
        should_reset_velocity: false,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Red,
        collision_damage: 10.0,
        hp: 30.0,
        engine_power: 12.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Enemy5>().unwrap();

    let mut response = Act1Response::new();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();

    if us_data.spit_projectile_start.is_none() && ctx.act1_ctx.get_randf64() < tick_len {
        us_data.spit_projectile_start = Some(SpitProjectileInfo { start: ctx.act1_ctx.get_room_time(), proj_spit: false});
    }

    if let Some(ref mut sps) = us_data.spit_projectile_start {
        if !sps.proj_spit && ctx.act1_ctx.get_room_time() - sps.start > 0.5 {
            sps.proj_spit = true;
            let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
            let self_as_weak = ctx.act1_ctx.get_self_as_weak();
            let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
            for i in 0..4 {
                let angle = (i as f64) * 1.0/2.0 * std::f64::consts::PI;
                let proj_speed = 12.0;
                let shape = Proj2Shape::TriFan {
                    center: Point::new(0.0, 0.0), 
                    vertexes: us_data.inner.vertexes.clone(),
                };
                let proj = Projectile2Builder::new(
                    Projectile2BuilderReq{
                        team: Team::Enemy,
                        damage_color: DamageColor::Red,
                        damage: 3.0,
                        owner: self_as_weak.clone(),
                        velocity_x: proj_speed * f64::cos(angle),
                        velocity_y: proj_speed * f64::sin(angle),
                        xform,
                        shape,
                        color: PROJ_COLOR,
                    }
                ).build(&mut nfo_ctx);
                response.add_room_obj(Rc::new(RefCell::new(proj)));
            }
        }
        if ctx.act1_ctx.get_room_time() - sps.start > 1.0 {
            us_data.spit_projectile_start = None;
        }
    }

    match us_data.spit_projectile_start {
        Some(_) => {
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Decelerate);
        }
        None => {
            let theta = (us_data.translate_dir as f64) * 1.0/2.0 * std::f64::consts::PI;
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: f64::cos(theta), ay: f64::sin(theta)});
        }
    }

    if us_data.should_reset_velocity {
        us_data.should_reset_velocity = false;
        ctx.su_ctx.su_common.set_translate_move(TranslateMove::ResetVelocity);
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Enemy5>().unwrap();
    assert!(us_data.border.vertexes.len() == 4);
    assert!(us_data.outer.vertexes.len() == 4);
    assert!(us_data.inner.vertexes.len() == 4);
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let border = xform.get_transformed_polygon(&us_data.border).vertexes;
    let outer = xform.get_transformed_polygon(&us_data.outer).vertexes;
    let inner = xform.get_transformed_polygon(&us_data.inner).vertexes;
    for i in 0..4 {
        let quad = [
            outer[i],
            border[i],
            border[(i+1)%4],
            outer[(i+1)%4],
        ];
        let dop = ctx.draw_ctx.do_tri_fan(DrawContext::COLOR_NSU_BORDER, &quad);
        ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
    }

    for i in 0..4 {
        let quad = [
            inner[i],
            outer[i],
            outer[(i+1)%4],
            inner[(i+1)%4],
        ];
        let dop = ctx.draw_ctx.do_tri_fan(OUTER_COLOR, &quad);
        ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
    }

    let inner_color = match us_data.spit_projectile_start {
        Some(ref x) => Color::lerp(PROJ_COLOR, INNER_COLOR, 2.0 * f64::abs(0.5 - (ctx.draw_ctx.get_room_time() - x.start)) as f32),
        None => INNER_COLOR,
    };
    let dop = ctx.draw_ctx.do_tri_fan(inner_color, &inner);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop);
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Enemy5>().unwrap();
    if !ctx.hc_ctx.is_other_spectral() {
        us_data.should_reset_velocity = true;
        us_data.translate_dir = ctx.hc_ctx.get_randi64(0..4);
    }
    HandleCollisionResponse::new()
}