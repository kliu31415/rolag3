use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, RofizObjType}, standard_unit_common::TranslateMove}, damage::DamageColor, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point, Vector}, util::{rotate_polygon, regular_polygon, get_inner_polygon}}, util::lerp::lerp_f64};

/* SquareRgb2Tri moves along a wall and shoots pairs of rotating homing projectiles. The rotating homing projectiles
   can be of different colors than the square itself.
*/

const IDX_TO_DAMAGE_COLOR: [DamageColor; 3] = [
    DamageColor::Red,
    DamageColor::Green,
    DamageColor::Blue,
];

const IDX_TO_OUTER_DRAW_COLOR: [Color; 3] = [
    Color::new(0.2, 0.01, 0.01, 1.0),
    Color::new(0.01, 0.2, 0.01, 1.0),
    Color::new(0.02, 0.02, 0.2, 1.0),
];

const IDX_TO_INNER_DRAW_COLOR: [Color; 3] = [
    Color::new(5.0, 0.01, 0.01, 1.0),
    Color::new(0.01, 1.6, 0.01, 1.0),
    Color::new(0.05, 0.05, 14.5, 1.0),
];

const INNER_CIRCLE_RADIUS: f32 = 0.25;

const INNER_CIRCLE_CENTERS: [Point; 2] = [
    Point::new(0.3, -0.28),
    Point::new(0.3, 0.28),
];

const PROJ_FIRE_INTERVAL: f64 = 0.0025;
const PROJ_SPEED: f64 = 180.0;
const ALTERNATE_EVERY_N: usize = 400;
const START_FIRE_PROJ_NUM: usize = 500;

fn damage_color_to_idx(dc: DamageColor) -> usize {
    match dc {
        DamageColor::Red => 0,
        DamageColor::Green => 1,
        DamageColor::Blue => 2,
        _ => panic!("can't convert DamageColor({:?}) to index", dc),
    }
}

struct SquareRgb2Circle {
    border_vertexes: [Point; 4],
    inner_vertexes: [Point; 4],
    outer_color: usize,
    inner_colors: [usize; 2],
    position_fn: Box<dyn Fn(f64) -> (f64, f64)>,
    num_proj_fired: usize,
}


pub fn position_fn_between_two_points(time1to2: f64, (x1, y1): (f64, f64), (x2, y2): (f64, f64)) -> Box<dyn Fn(f64) -> (f64, f64)> {
    Box::new(move |age: f64| -> (f64, f64) {
        let age = age % (2.0 * time1to2);
        if age < time1to2 {
            (lerp_f64(x1, x2, age / time1to2), lerp_f64(y1, y2, age / time1to2))
        } else {
            let age = 2.0 * time1to2 - age;
            (lerp_f64(x1, x2, age / time1to2), lerp_f64(y1, y2, age / time1to2))
        }
    })
}

pub fn new_square_rgb_2circle(
    ctx: &mut NewRoomObjectContext, 
    outer_color: DamageColor,
    inner_colors: [DamageColor; 2],
    position_fn: Box<dyn Fn(f64) -> (f64, f64)>,
    theta: f64,
) -> StandardUnit1 {
    let mut border_vertexes: [Point; 4] = regular_polygon(4, 1.3)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let inner_vertexes: [Point; 4] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let (x, y) = (position_fn)(0.0);
    let xform = Transformation::new(x, y, theta);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = SquareRgb2Circle {
        border_vertexes,
        inner_vertexes,
        outer_color: damage_color_to_idx(outer_color),
        inner_colors: inner_colors.map(|x| damage_color_to_idx(x)),
        position_fn,
        num_proj_fired: 0,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: outer_color,
        hp: 20.0,
        engine_power: 0.0, // nop
        tire_traction: 0.0, // nop
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgb2Circle>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let (x, y) = (us_data.position_fn)(unit_age);
    ctx.su_ctx.su_common.set_translate_move(TranslateMove::SetXY { x, y });

    let desired_npf = (unit_age / PROJ_FIRE_INTERVAL) as usize;
    while us_data.num_proj_fired < desired_npf {
        let fired_time_ago = unit_age - PROJ_FIRE_INTERVAL * (us_data.num_proj_fired as f64);
        let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());
        for i in 0..2 {
            if i == 1 && us_data.num_proj_fired < START_FIRE_PROJ_NUM {
                // before START_FIRE_AT, only fire a prelude for one of the lasers. The only laser doesn't fire at all.
                continue;
            }
            let laser_fired = if us_data.num_proj_fired < START_FIRE_PROJ_NUM {
                true
            } else {
                ((us_data.num_proj_fired - START_FIRE_PROJ_NUM) / ALTERNATE_EVERY_N) % 2 == i
            };
            let self_as_weak = ctx.act1_ctx.get_self_as_weak();
            let inner_circle_center = INNER_CIRCLE_CENTERS[i].rotated(xform.dtheta as f32);
            let velocity_x = PROJ_SPEED * f64::cos(xform.dtheta);
            let velocity_y = PROJ_SPEED * f64::sin(xform.dtheta);
            let proj_xform = Transformation::new(
                xform.dx + inner_circle_center.x as f64 + fired_time_ago * velocity_x,
                xform.dy + inner_circle_center.y as f64 + fired_time_ago * velocity_y, 
                xform.dtheta,
            );
            let (damage, draw_color) = if !laser_fired || us_data.num_proj_fired < START_FIRE_PROJ_NUM {
                let mut color = IDX_TO_INNER_DRAW_COLOR[us_data.inner_colors[i]];
                color.a = 0.01;
                (0.0, color)
            } else {
                (10.0 * PROJ_FIRE_INTERVAL, IDX_TO_INNER_DRAW_COLOR[us_data.inner_colors[i]])
            };
            let proj = Projectile2Builder::new(Projectile2BuilderReq {
                team: Team::Enemy,
                damage_color: IDX_TO_DAMAGE_COLOR[us_data.inner_colors[i]],
                damage,
                owner: self_as_weak,
                velocity_x,
                velocity_y,
                xform: proj_xform,
                shape: Proj2Shape::Circle { x: 0.0, y: 0.0, r: INNER_CIRCLE_RADIUS },
                color: draw_color,
            }).build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
        us_data.num_proj_fired += 1;
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgb2Circle>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(IDX_TO_OUTER_DRAW_COLOR[us_data.outer_color]);
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let xlate_vec = Vector::new(xform.dx as f32, xform.dy as f32);
    let border_vertexes = us_data.border_vertexes.map(|v| v.rotated(xform.dtheta as f32).translated(xlate_vec));
    let outer_vertexes = us_data.inner_vertexes.map(|v| v.rotated(xform.dtheta as f32).translated(xlate_vec));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let outer_dop = ctx.draw_ctx.do_quad_fan(outer_color, outer_vertexes);

    let inner_dops: [_; 2] = std::array::from_fn(|i| {
        let color = IDX_TO_INNER_DRAW_COLOR[us_data.inner_colors[i]];
        let center = INNER_CIRCLE_CENTERS[i].rotated(xform.dtheta as f32).translated(xlate_vec);
        ctx.draw_ctx.do_circle(color, center, INNER_CIRCLE_RADIUS)
    });

    let dops = [border_dop].into_iter().chain([outer_dop].into_iter()).chain(inner_dops.into_iter());
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(dops.collect()));
}