use std::{cell::RefCell, rc::Rc};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Act1Response, Team, Act1QueryArgs, Act1QueryResult}, damage::DamageColor, unit::{standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, StandardUnit1}, standard_unit_common::TranslateMove}, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point, Vector}, util::{regular_polygon, get_inner_polygon, rotate_polygon}}};

/* SmallOctagonRgbCircle sprays out a laser that "rotates" among its 8 sides
   It erratically moves, biased towards the player's direction
*/

const OUTER_COLORS: [Color; 3] = [
    Color::new(0.1, 0.001, 0.001, 1.0),
    Color::new(0.001, 0.07, 0.001, 1.0),
    Color::new(0.001, 0.001, 0.1, 1.0),
];

const PROJ_COLORS: [Color; 3] = [
    Color::new(5.0, 0.06, 0.06, 1.0),
    Color::new(0.06, 1.5, 0.06, 1.0),
    Color::new(0.06, 0.06, 14.0, 1.0),
];

const PROJ_RADIUS: f32 = 0.25;
const PROJ_FIRE_INTERVAL: f64 = 0.0025;
const PROJ_SPEED: f64 = 180.0;

const LASER_START_ROOM_PRELUDE_DURATION: f64 = 1.25;
const LASER_DURATION_EACH: f64 = 0.5;

pub struct SmallOctRgbCircle {
    xlate_dir_qr: Option<Rc<RefCell<Act1QueryResult>>>,
    border_vertexes: [Point; 8],
    outer_vertexes: [Point; 8],
    xlate_action: XlateAction,
    laser_action: LaserAction,
}

struct XlateAction {
    // before start_xlate_at, the unit doesn't move
    start_xlate_at: f64, 
    direction: i64, 
    end_age: f64,
}

struct LaserAction {
    laser_dir: i64,
    next_dir_at: f64,
    num_proj_fired: u64,
}

pub fn new_small_octagon_rgb_circle(
    ctx: &mut NewRoomObjectContext, 
    damage_color: DamageColor, 
    x: f64, 
    y: f64,
) -> StandardUnit1 {
    let mut border_vertexes: [Point; 8] = regular_polygon(8, 0.7)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_8, &mut border_vertexes);
    let outer_vertexes: [Point; 8] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = SmallOctRgbCircle { 
        xlate_dir_qr: None,
        border_vertexes,
        outer_vertexes,
        xlate_action: XlateAction { 
            start_xlate_at: 0.0, 
            direction: ctx.get_rng().gen_i64_range(0..8), 
            end_age: 0.0,
        },
        laser_action: LaserAction {
            laser_dir: ctx.get_rng().gen_i64_range(0..8),
            next_dir_at: LASER_START_ROOM_PRELUDE_DURATION + LASER_DURATION_EACH,
            num_proj_fired: 0,
        },
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: 20.0,
        engine_power: 20.0,
        tire_traction: 50.0,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SmallOctRgbCircle>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());

    if let Some(qr) = us_data.xlate_dir_qr.take() {
        let Act1QueryResult::ClosestUnit(cu_opt) = &*qr.borrow() else {panic!("unexpected qr={:?}", &*qr.borrow())};
        let mut probabilities = [1.2; 8];
        if let Some(cu) = cu_opt {
            let dx = cu.x - xform.dx;
            let dy = cu.y - xform.dy;
            let a2p = f64::atan2(dy, dx);
            for i in 0..8 {
                let candidate = xform.dtheta + (i as f64) * std::f64::consts::FRAC_PI_4;
                let dot = f64::cos(candidate) * f64::cos(a2p) + f64::sin(candidate) * f64::sin(a2p);
                probabilities[i] += dot;
            }
        }
        let start_xlate_at = unit_age + ctx.act1_ctx.get_rng().gen_f64_range(0.1 .. 0.4);
        let end_age = start_xlate_at + if ctx.act1_ctx.get_rng().gen_bernoulli(0.1) {
            ctx.act1_ctx.get_rng().gen_f64_range(0.6 .. 0.7)
        } else {
            ctx.act1_ctx.get_rng().gen_f64_range(0.15 .. 0.3)
        };
        us_data.xlate_action = XlateAction { 
            direction: ctx.act1_ctx.get_rng().sample_weighted_slice_f64(&probabilities) as i64,
            start_xlate_at,
            end_age,
        };
    }

    let mut response = Act1Response::new();

    if unit_age < us_data.xlate_action.start_xlate_at {
        ctx.su_ctx.su_common.set_translate_move(TranslateMove::Decelerate);
    } else {
        let angle = us_data.xlate_action.direction as f64 * std::f64::consts::FRAC_PI_4;
        let movement = TranslateMove::Accelerate { ax: f64::cos(angle), ay: f64::sin(angle) };
        ctx.su_ctx.su_common.set_translate_move(movement);
    }
    if unit_age > us_data.xlate_action.end_age {
        let query = Act1QueryArgs::ClosestUnit { x: xform.dx, y: xform.dy, team_filter: Some(Team::Player) };
        us_data.xlate_dir_qr = Some(response.add_query(query));
    }

    let owner_age = ctx.su_ctx.su_common.get_unit_time();
    let desired_npf = (owner_age / PROJ_FIRE_INTERVAL) as u64;
    while us_data.laser_action.num_proj_fired < desired_npf {
        let fired_ago = owner_age - us_data.laser_action.num_proj_fired as f64 * PROJ_FIRE_INTERVAL;
        let base_angle = us_data.laser_action.laser_dir as f64 * std::f64::consts::FRAC_PI_4;
        for i in [0, 1] {
            if i == 1 && owner_age < LASER_START_ROOM_PRELUDE_DURATION {
                continue;
            }
            let angle = base_angle + i as f64 * std::f64::consts::FRAC_PI_4;
            let proj_xform = Transformation::new(
                xform.dx + fired_ago * PROJ_SPEED * f64::cos(angle), 
                xform.dy + fired_ago * PROJ_SPEED * f64::sin(angle), 
                0.0,
            );
            let is_prelude = if owner_age < LASER_START_ROOM_PRELUDE_DURATION {
                true
            } else {
                i == 1
            };
            let color = if is_prelude {
                let mut color = PROJ_COLORS[ctx.su_ctx.damage_color.to_rgb_idx()];
                color.a = 0.01;
                color
            } else {
                PROJ_COLORS[ctx.su_ctx.damage_color.to_rgb_idx()]
            };
            let self_as_weak = ctx.act1_ctx.get_self_as_weak();
            let proj = Projectile2Builder::new(Projectile2BuilderReq {
                team: Team::Enemy,
                damage_color: *ctx.su_ctx.damage_color,
                damage: if is_prelude {0.0} else {3.0},
                owner: self_as_weak,
                velocity_x: PROJ_SPEED * f64::cos(angle),
                velocity_y: PROJ_SPEED * f64::sin(angle),
                xform: proj_xform,
                shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: PROJ_RADIUS },
                color,
            }).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
        us_data.laser_action.num_proj_fired += 1;
    }

    if owner_age > us_data.laser_action.next_dir_at {
        us_data.laser_action.laser_dir += 1;
        us_data.laser_action.laser_dir %= 8;
        us_data.laser_action.next_dir_at += LASER_DURATION_EACH;
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SmallOctRgbCircle>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let color_idx = ctx.su_ctx.damage_color.to_rgb_idx();
    let outer_color = ctx.su_ctx.su_common.get_draw_color(OUTER_COLORS[color_idx]);
    let inner_color = ctx.su_ctx.su_common.get_draw_color(PROJ_COLORS[color_idx]);
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let xlate = Vector::new(xform.dx as f32, xform.dy as f32);
    let border_vertexes = us_data.border_vertexes.map(|p| p.rotated(xform.dtheta as f32).translated(xlate));
    let outer_vertexes = us_data.outer_vertexes.map(|p| p.rotated(xform.dtheta as f32).translated(xlate));

    let mut draw_ops = Vec::new();
    draw_ops.push(ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes));
    draw_ops.push(ctx.draw_ctx.do_tri_fan(outer_color, &outer_vertexes));
    draw_ops.push(ctx.draw_ctx.do_circle(inner_color, Point::new(xform.dx as f32, xform.dy as f32), PROJ_RADIUS));
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(draw_ops.into()));
}