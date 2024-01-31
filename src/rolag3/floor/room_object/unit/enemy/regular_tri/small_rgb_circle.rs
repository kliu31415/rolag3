use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1}, standard_unit_common::TranslateMove}, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point, Vector}, util::{regular_polygon, get_inner_polygon}}};

/* RegtriSmallRgbCircle sprays out waves of three projectiles.
   It erratically moves, biased towards the player's direction
*/

const OUTER_COLORS: [Color; 3] = [
    Color::new(0.1, 0.001, 0.001, 1.0),
    Color::new(0.001, 0.1, 0.001, 1.0),
    Color::new(0.001, 0.001, 0.1, 1.0),
];

const INNER_COLORS: [Color; 3] = [
    Color::new(1.0, 0.01, 0.01, 1.0),
    Color::new(0.01, 1.0, 0.01, 1.0),
    Color::new(0.01, 0.01, 1.0, 1.0),
];

const PROJ_COLORS: [Color; 3] = [
    Color::new(5.0, 0.06, 0.06, 1.0),
    Color::new(0.06, 1.5, 0.06, 1.0),
    Color::new(0.06, 0.06, 14.0, 1.0),
];

const PROJ_RADIUS: f32 = 0.25;

pub struct RegtriSmallRgbCircle {
    xlate_dir_qr: Option<Rc<RefCell<Act1QueryResult>>>,
    border_vertexes: [Point; 3],
    outer_vertexes: [Point; 3],
    translate_dir: Option<i64>,
    last_fired_at: Option<f64>,
    next_fire_at: Option<f64>,
}

pub fn new_regtri_small_rgb_circle(ctx: &mut NewRoomObjectContext, damage_color: DamageColor, x: f64, y: f64) -> StandardUnit1 {
    let border_vertexes: [Point; 3] = regular_polygon(3, 0.9)[..].try_into().unwrap();
    let outer_vertexes: [Point; 3] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, ctx.get_rng().gen_f64_range(0.0 .. (2.0 * std::f64::consts::PI)));
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = RegtriSmallRgbCircle { 
        xlate_dir_qr: None,
        border_vertexes,
        outer_vertexes,
        translate_dir: None,
        last_fired_at: None,
        next_fire_at: None,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: 15.0,
        engine_power: 11.0,
        tire_traction: 30.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RegtriSmallRgbCircle>().unwrap();
    let tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());

    if let Some(qr) = us_data.xlate_dir_qr.take() {
        let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected qr={:?}", &*qr.borrow())};
        let mut probabilities = [1.5, 1.5, 1.5];
        if let Some(cu) = cu_opt {
            let dx = cu.x - xform.dx;
            let dy = cu.y - xform.dy;
            let a2p = f64::atan2(dy, dx);
            for i in 0..3 {
                let candidate = xform.dtheta + (i as f64) * 2.0 / 3.0 * std::f64::consts::PI;
                let dot = f64::cos(candidate) * f64::cos(a2p) + f64::sin(candidate) * f64::sin(a2p);
                probabilities[i] += dot;
            }
        }
        us_data.translate_dir = Some(ctx.act1_ctx.get_rng().sample_weighted_slice_f64(&probabilities) as i64);
    }

    let mut response = Act1Response::new();
    if ctx.act1_ctx.get_rng().gen_bernoulli(3.0 * tick_len) {
        if ctx.act1_ctx.get_rng().gen_bernoulli(0.1) {
            us_data.translate_dir = None;
        } else {
            let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
            us_data.xlate_dir_qr = Some(response.add_query(query));
        }
    }

    if let Some(nfa) = us_data.next_fire_at {
        if unit_age >= nfa {
            let self_as_weak = ctx.act1_ctx.get_self_as_weak();
            for i in 0..3 {
                let proj_velocity = 10.0;
                let angle = i as f64 * 2.0 / 3.0 * std::f64::consts::PI;
                let proj = Projectile2Builder::new(Projectile2BuilderReq {
                    team: Team::Enemy,
                    damage_color: *ctx.su_ctx.damage_color,
                    damage: 3.0,
                    owner: self_as_weak.clone(),
                    velocity_x: proj_velocity * f64::cos(angle),
                    velocity_y: proj_velocity * f64::sin(angle),
                    xform,
                    shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: PROJ_RADIUS },
                    color: PROJ_COLORS[ctx.su_ctx.damage_color.to_rgb_idx()],
                }).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
                response.add_room_obj(Rc::new(RefCell::new(proj)));
            }
            us_data.last_fired_at = Some(nfa);
            us_data.next_fire_at = None;
        }
    } else if ctx.act1_ctx.get_rng().gen_bernoulli(1.0 * tick_len) {
        us_data.next_fire_at = Some(unit_age + 0.3);
    }

    if let Some(td) = us_data.translate_dir {
        if us_data.last_fired_at.is_none() || (unit_age - us_data.last_fired_at.unwrap()) > 0.2 {
            let angle = xform.dtheta + td as f64 * 2.0 / 3.0 * std::f64::consts::PI;
            let movement = TranslateMove::Accelerate { ax: f64::cos(angle), ay: f64::sin(angle) };
            ctx.su_ctx.su_common.set_translate_move(movement);
        } else {
            ctx.su_ctx.su_common.set_translate_move(TranslateMove::Decelerate);
        }
    } else {
        ctx.su_ctx.su_common.set_translate_move(TranslateMove::Decelerate);
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RegtriSmallRgbCircle>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let color_idx = ctx.su_ctx.damage_color.to_rgb_idx();
    let outer_color = ctx.su_ctx.su_common.get_draw_color(OUTER_COLORS[color_idx]);
    let mut lerp_t = 1.0;
    if let Some(lfa) = us_data.last_fired_at {
        lerp_t = f64::min(lerp_t, 5.0 * (unit_age - lfa));
    }
    if let Some(nfa) = us_data.next_fire_at {
        lerp_t = f64::min(lerp_t, 5.0 * (nfa - unit_age));
    }
    let inner_color = Color::lerp(PROJ_COLORS[color_idx], INNER_COLORS[color_idx], lerp_t as f32);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(inner_color);
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let xlate = Vector::new(xform.dx as f32, xform.dy as f32);
    let border_vertexes = us_data.border_vertexes.map(|p| p.rotated(xform.dtheta as f32).translated(xlate));
    let outer_vertexes = us_data.outer_vertexes.map(|p| p.rotated(xform.dtheta as f32).translated(xlate));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let outer_dop = ctx.draw_ctx.do_tri(outer_color, outer_vertexes);
    let inner_dop = ctx.draw_ctx.do_circle(inner_color, Point::new(xform.dx as f32, xform.dy as f32), PROJ_RADIUS);
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(Box::new([border_dop, outer_dop, inner_dop])));
}