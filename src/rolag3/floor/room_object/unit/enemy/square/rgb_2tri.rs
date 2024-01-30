use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, RofizObjType}, standard_unit_common::TranslateMove}, damage::DamageColor, projectile::projectile2::{Projectile2Builder, Projectile2BuilderReq, Proj2Shape}}, rofiz::rofiz_object::Transformation, draw::{Color, DrawContext}}, geometry::{shape::{Shape, Point, Vector}, util::{rotate_polygon, regular_polygon, get_inner_polygon}}};

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

const INNER_TRI_VERTEXES: [Point; 3] = [
    Point::new(0.3, 0.0),
    Point::new(-0.3, -0.2),
    Point::new(-0.3, 0.2),
];

const INNER_TRI_OFFSETS: [Vector; 2] = [
    Vector::new(0.35, -0.28),
    Vector::new(0.35, 0.28),
];

const FIRE_PROJ_INTERVAL: f64 = 0.5;

fn damage_color_to_idx(dc: DamageColor) -> usize {
    match dc {
        DamageColor::Red => 0,
        DamageColor::Green => 1,
        DamageColor::Blue => 2,
        _ => panic!("can't convert DamageColor({:?}) to index", dc),
    }
}

struct SquareRgb2Tri {
    border_vertexes: [Point; 4],
    inner_vertexes: [Point; 4],
    outer_color: usize,
    inner_colors: [Option<usize>; 2],
    position_fn: Box<dyn Fn(f64) -> (f64, f64)>,
    fired_proj_time_ago: f64,
}

pub fn new_square_rgb_2tri(
    ctx: &mut NewRoomObjectContext, 
    outer_color: DamageColor,
    inner_colors: [Option<DamageColor>; 2],
    position_fn: Box<dyn Fn(f64) -> (f64, f64)>,
    theta: f64,
) -> StandardUnit1 {
    let mut border_vertexes: [Point; 4] = regular_polygon(4, 1.1)[..].try_into().unwrap();
    rotate_polygon(std::f32::consts::FRAC_PI_4, &mut border_vertexes);
    let inner_vertexes: [Point; 4] = get_inner_polygon(0.1, &border_vertexes)[..].try_into().unwrap();
    let (x, y) = (position_fn)(0.0);
    let xform = Transformation::new(x, y, theta);
    let shape = Shape::of_polygon(Box::new(border_vertexes));
    let us_data = SquareRgb2Tri {
        border_vertexes,
        inner_vertexes,
        outer_color: damage_color_to_idx(outer_color),
        inner_colors: inner_colors.map(|x| x.map(|y| damage_color_to_idx(y))),
        position_fn,
        fired_proj_time_ago: 0.0,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: outer_color,
        hp: 20.0,
        engine_power: 0.0, //nop,
        tire_traction: 0.0, //nop,
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, shape)
        .us_data(Box::new(us_data))
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let mut response = Act1Response::new();
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgb2Tri>().unwrap();
    let unit_age = ctx.su_ctx.su_common.get_unit_time();
    let unit_tick_len = ctx.su_ctx.su_common.get_unit_tick_len();
    let (x, y) = (us_data.position_fn)(unit_age);
    ctx.su_ctx.su_common.set_translate_move(TranslateMove::SetXY { x, y });

    us_data.fired_proj_time_ago += unit_tick_len;
    if us_data.fired_proj_time_ago > FIRE_PROJ_INTERVAL {
        us_data.fired_proj_time_ago -= FIRE_PROJ_INTERVAL;
        let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.act1_ctx.get_rofiz());
        for i in 0..2 {
            let Some(inner_color) = us_data.inner_colors[i] else {continue;};
            let self_as_weak = ctx.act1_ctx.get_self_as_weak();
            let proj_speed = 10.0;
            let inner_tri_offset = INNER_TRI_OFFSETS[i].rotated(xform.dtheta as f32);
            let proj_xform = Transformation::new(
                xform.dx + inner_tri_offset.x as f64, 
                xform.dy + inner_tri_offset.y as f64, 
                xform.dtheta,
            );
            let rotate_homing_fn = Box::new(|_: f64| -> f64 {
                0.3
            });
            let proj = Projectile2Builder::new(Projectile2BuilderReq {
                team: Team::Enemy,
                damage_color: IDX_TO_DAMAGE_COLOR[inner_color],
                damage: 3.0,
                owner: self_as_weak,
                velocity_x: proj_speed * f64::cos(xform.dtheta),
                velocity_y: proj_speed * f64::sin(xform.dtheta),
                xform: proj_xform,
                shape: Proj2Shape::TriFan{center: Point::new(0.0, 0.0), vertexes: Box::new(INNER_TRI_VERTEXES) },
                color: IDX_TO_INNER_DRAW_COLOR[inner_color],
            }).homing_rotate_to_enemies_speed_fn(Box::new(rotate_homing_fn))
                .build(&mut NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx));
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
    }

    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SquareRgb2Tri>().unwrap();
    let border_color = ctx.su_ctx.su_common.get_draw_color(DrawContext::COLOR_NSU_BORDER);
    let outer_color = ctx.su_ctx.su_common.get_draw_color(IDX_TO_OUTER_DRAW_COLOR[us_data.outer_color]);
    let xform = ctx.su_ctx.su_common.get_rofiz_xform(ctx.draw_ctx.get_rofiz());
    let xlate_vec = Vector::new(xform.dx as f32, xform.dy as f32);
    let border_vertexes = us_data.border_vertexes.map(|v| v.rotated(xform.dtheta as f32).translated(xlate_vec));
    let outer_vertexes = us_data.inner_vertexes.map(|v| v.rotated(xform.dtheta as f32).translated(xlate_vec));
    let border_dop = ctx.draw_ctx.do_thick_border(border_color, &border_vertexes, &outer_vertexes);
    let outer_dop = ctx.draw_ctx.do_quad_fan(outer_color, outer_vertexes);

    let inner_dops: [_; 2] = std::array::from_fn(|i| {
        let color = match us_data.inner_colors[i] {
            Some(color) => IDX_TO_INNER_DRAW_COLOR[color],
            None => Color::new(0.0, 0.0, 0.0, 0.0),
        };
        let vertexes = INNER_TRI_VERTEXES
            .map(|v| v.translated(INNER_TRI_OFFSETS[i]).rotated(xform.dtheta as f32).translated(xlate_vec));
        ctx.draw_ctx.do_tri(color, vertexes)
    });

    let dops = [border_dop].into_iter().chain([outer_dop].into_iter()).chain(inner_dops.into_iter());
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, ctx.draw_ctx.dop_group(dops.collect()));
}