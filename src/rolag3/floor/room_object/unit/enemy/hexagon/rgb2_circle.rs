use std::{rc::Rc, cell::RefCell};

use crate::{geometry::{shape::{Point, Shape}, util::{regular_polygon, get_inner_polygon}}, rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response, HandleCollisionResponse}, unit::{standard_unit1::{StandardUnit1, StandardUnit1BuilderReq, StandardUnit1Builder, HandleCollisionLogic, SuAct1Context, SuDrawContext, SuHandleCollisionContext}, standard_unit_common::TranslateMove}, damage::DamageColor, projectile::projectile2::{Projectile2BuilderReq, Projectile2Builder, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}};

/* HexagonRgb2Circle randomly translates in one of 6 directions and periodically shoots a radial wave of 16 projectiles.
   Variant: BigHexagonRgb2Circle acts the same, except it has a potentially different outer color and spawns a regular
   HexagonRgb2Circle on death.
*/

const OUTER_COLORS: [Color; 3] = [
    Color::new(0.2, 0.05, 0.05, 1.0),
    Color::new(0.05, 0.2, 0.05, 1.0),
    Color::new(0.05, 0.05, 0.2, 1.0),
];

const INNER_COLORS: [Color; 3] = [
    Color::new(1.0, 0.1, 0.1, 1.0),
    Color::new(0.1, 1.0, 0.1, 1.0),
    Color::new(0.1, 0.1, 1.0, 1.0),
];

const PROJ_COLORS: [Color; 3] = [
    Color::new(5.0, 0.05, 0.05, 1.0),
    Color::new(0.05, 1.5, 0.05, 1.0),
    Color::new(0.05, 0.05, 14.5, 1.0),
];

const PROJ_RADIUS: f32 = 0.2;
const PROJ_SPEED: f64 = 10.0;

const REMOVE_WRAPPER_HP: f64 = 20.0;
const INNER_ENGINE_POWER: f64 = 4.0;

struct HexagonRgb2Circle {
    wrapper_border_vertexes: [Point; 6],
    wrapper_outer_vertexes: [Point; 6],

    border_vertexes: [Point; 6],
    outer_vertexes: [Point; 6],
    // no inner_vertexes field because inner shape is a circle

    wrapper_damage_color: DamageColor,
    inner_damage_color: DamageColor,

    since_last_fired: f64,
    fire_interval: f64,
    translate_dir: i64,
}

pub fn new_hexagon_rgb2_circle(
    ctx: &mut NewRoomObjectContext, 
    damage_color: DamageColor,
    x: f64, 
    y: f64,
) -> StandardUnit1 {
    let wrapper_border_vertexes: [Point; 6] = regular_polygon(6, 1.2)[..].try_into().unwrap();
    let wrapper_outer_vertexes: [Point; 6] = get_inner_polygon(0.1, &wrapper_border_vertexes)[..].try_into().unwrap();
    let border_vertexes: [Point; 6] = regular_polygon(6, 0.9)[..].try_into().unwrap();
    let outer_vertexes: [Point; 6] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = HexagonRgb2Circle {
        wrapper_border_vertexes,
        wrapper_outer_vertexes,
        border_vertexes,
        outer_vertexes,
        wrapper_damage_color: DamageColor::NotSet,
        inner_damage_color: damage_color,
        since_last_fired: 0.0,
        fire_interval: 0.9 + 0.1 * ctx.get_randf64(),
        translate_dir: ctx.get_randi64(0..6),
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: REMOVE_WRAPPER_HP,
        engine_power: INNER_ENGINE_POWER,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

pub fn new_big_hexagon_rgb2_circle(
    ctx: &mut NewRoomObjectContext, 
    wrapper_damage_color: DamageColor,
    inner_damage_color: DamageColor,
    x: f64, 
    y: f64,
) -> StandardUnit1 {
    let wrapper_border_vertexes: [Point; 6] = regular_polygon(6, 1.3)[..].try_into().unwrap();
    let wrapper_outer_vertexes: [Point; 6] = get_inner_polygon(0.1, &wrapper_border_vertexes)[..].try_into().unwrap();
    let border_vertexes: [Point; 6] = regular_polygon(6, 0.9)[..].try_into().unwrap();
    let outer_vertexes: [Point; 6] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = HexagonRgb2Circle {
        wrapper_border_vertexes,
        wrapper_outer_vertexes,
        border_vertexes,
        outer_vertexes,
        wrapper_damage_color,
        inner_damage_color,
        since_last_fired: 0.0,
        fire_interval: 0.9 + 0.1 * ctx.get_randf64(),
        translate_dir: ctx.get_randi64(0..6),
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: wrapper_damage_color,
        hp: 40.0,
        engine_power: 8.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .handle_collision_logic(HandleCollisionLogic::CustomFn(Box::new(handle_collision)))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<HexagonRgb2Circle>().unwrap();

    let mut response = Act1Response::new();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();

    if ctx.act1_ctx.get_rng().gen_bernoulli(0.5 * tick_len) {
        us_data.translate_dir = ctx.act1_ctx.get_randi64(0..6);
    }

    if ctx.su_ctx.su_common.get_cur_hp() <= REMOVE_WRAPPER_HP {
        *ctx.su_ctx.damage_color = us_data.inner_damage_color;
        ctx.su_ctx.su_common.set_engine_power(INNER_ENGINE_POWER);
    }

    us_data.since_last_fired += tick_len;
    if us_data.since_last_fired > us_data.fire_interval {
        us_data.since_last_fired -= us_data.fire_interval;
        let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());
        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
        let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
        let num_proj = 16;
        for i in 0..num_proj {
            let angle = (i as f64) / (num_proj as f64) * 2.0 * std::f64::consts::PI;
            let proj = Projectile2Builder::new(
                Projectile2BuilderReq {
                    team: Team::Enemy,
                    damage_color: us_data.inner_damage_color,
                    damage: 3.0,
                    owner: self_as_weak.clone(),
                    velocity_x: PROJ_SPEED * f64::cos(angle),
                    velocity_y: PROJ_SPEED * f64::sin(angle),
                    xform,
                    shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: PROJ_RADIUS },
                    color: PROJ_COLORS[us_data.inner_damage_color.to_rgb_idx()],
                }
            ).build(&mut nfo_ctx);
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
    }

    let theta = us_data.translate_dir as f64 * std::f64::consts::FRAC_PI_3;
    ctx.su_ctx.su_common.set_translate_move(TranslateMove::Accelerate { ax: f64::cos(theta), ay: f64::sin(theta)});

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<HexagonRgb2Circle>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let lerp_t = f64::min(1.0, 5.0 * f64::min(us_data.since_last_fired, us_data.fire_interval - us_data.since_last_fired));
    let inner_color = Color::lerp(INNER_COLORS[us_data.inner_damage_color.to_rgb_idx()], PROJ_COLORS[us_data.inner_damage_color.to_rgb_idx()], lerp_t as f32);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(inner_color);
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let border_vertexes = us_data.border_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let outer_vertexes = us_data.outer_vertexes.map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);

    let colored_outer_vertexes = [(Point::new(xform.dx as f32, xform.dy as f32), INNER_COLORS[us_data.inner_damage_color.to_rgb_idx()])]
        .into_iter()
        .chain(outer_vertexes.into_iter().chain([outer_vertexes[0]].into_iter())
            .map(|p| (p, Color::lerp(INNER_COLORS[us_data.inner_damage_color.to_rgb_idx()], 
                OUTER_COLORS[us_data.inner_damage_color.to_rgb_idx()], lerp_t as f32))).into_iter())
        .collect::<Box<_>>();
    let outer_dop = ctx.draw_ctx.do_tri_fan_multicolor(&colored_outer_vertexes);
    let inner_dop = ctx.draw_ctx.do_circle(inner_color, Point::new(xform.dx as f32, xform.dy as f32), PROJ_RADIUS);
    let mut dops = vec![border_dop, outer_dop, inner_dop];

    if ctx.su_ctx.su_common.get_cur_hp() > REMOVE_WRAPPER_HP {
        let wrapper_border_vertexes = us_data.wrapper_border_vertexes
            .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
        let wrapper_outer_vertexes = us_data.wrapper_outer_vertexes
            .map(|v| Point::new(xform.dx as f32 + v.x, xform.dy as f32 + v.y));
        dops.push(ctx.draw_ctx.do_thick_border(border_color, &wrapper_border_vertexes, &wrapper_outer_vertexes));
        let inner_color = ctx.su_ctx.su_common.get_draw_color(INNER_COLORS[us_data.wrapper_damage_color.to_rgb_idx()]);
        dops.push(ctx.draw_ctx.do_thick_border(inner_color, &wrapper_outer_vertexes, &outer_vertexes));
    }


    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(dops.into()));
}

fn handle_collision(ctx: &mut SuHandleCollisionContext) -> HandleCollisionResponse {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<HexagonRgb2Circle>().unwrap();
    if !ctx.hc_ctx.is_other_spectral() {
        us_data.translate_dir = ctx.hc_ctx.get_randi64(0..6);
    }
    HandleCollisionResponse::new()
}