/* Circular Turret BluntStar3 fires projectiles of a given color
*/

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext}, standard_unit_common::RotateMove}, damage::DamageColor}, draw::{Color, DrawContext}, rofiz::rofiz_object::Transformation}, geometry::{shape::{Point, Shape}, star::get_blunt_star, util::get_inner_polygon}};

const CIRCLE_BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);
const TURRET_BORDER_COLOR: Color = Color::new(0.2, 0.2, 0.2, 1.0);

const CIRCLE_BORDER_RADIUS: f32 = 1.5;
const CIRCLE_INNER_RADIUS: f32 = 1.4;
const TURRET_INNER_RADIUS: f32 = 0.5;
const TURRET_OUTER_RADIUS: f32 = 1.2;
const TURRET_BORDER_THICKNESS: f32 = 0.1;

struct CirTurBluntStar3 {
    circle_inner_color: Color,
    turret_inner_color: Color,
    turret_border_vertexes: Shape,
    turret_inner_vertexes: Shape,
    turret_border_xformed_cache: Shape,
    turret_inner_xformed_cache: Shape,
}

pub fn new_circular_turret_bluntstar3(ctx: &mut NewRoomObjectContext, x: f64, y: f64, color: DamageColor) -> StandardUnit1 {
    let turret_border_vertexes = get_blunt_star(3, TURRET_INNER_RADIUS, TURRET_OUTER_RADIUS, -std::f32::consts::FRAC_PI_3);
    let turret_inner_vertexes = get_inner_polygon(TURRET_BORDER_THICKNESS, &turret_border_vertexes);
    let xform = Transformation::new(x, y, 0.0);
    let circle_inner_color = match color {
        DamageColor::Red => Color::new(0.4, 0.1, 0.1, 1.0),
        DamageColor::Green => Color::new(0.1, 0.4, 0.1, 1.0),
        DamageColor::Blue => Color::new(0.1, 0.1, 0.4, 1.0),
        _ => panic!("can't create turret with color {:?}", color),
    };
    let turret_inner_color = match color {
        DamageColor::Red => Color::new(1.0, 0.1, 0.1, 1.0),
        DamageColor::Green => Color::new(0.1, 1.0, 0.1, 1.0),
        DamageColor::Blue => Color::new(0.1, 0.1, 1.0, 1.0),
        _ => panic!("can't create turret with color {:?}", color),
    };
    let us_data = CirTurBluntStar3 {
        circle_inner_color,
        turret_inner_color,
        turret_border_vertexes: Shape::of_polygon(turret_border_vertexes),
        turret_inner_vertexes: Shape::of_polygon(turret_inner_vertexes),
        turret_border_xformed_cache: Shape::default(),
        turret_inner_xformed_cache: Shape::default(),
    };

    // TODO: make turret undamageable
    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Blue,
        hp: 1000.0,
        engine_power: 1.0, // nop
        tire_traction: 1.0, // nop
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, Shape::of_circle(Point::new(0.0, 0.0), CIRCLE_BORDER_RADIUS))
        .us_data(Box::new(us_data))
        .angular_power(1.0)
        .angular_traction(10.0)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate { atheta: 1.0 });
    Act1Response::new()
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<CirTurBluntStar3>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let circles_dop = ctx.draw_ctx.do_concentric_circle(
        us_data.circle_inner_color,
        CIRCLE_BORDER_COLOR, 
        Point::new(xform.dx as f32, xform.dy as f32), 
        CIRCLE_INNER_RADIUS, 
        CIRCLE_BORDER_RADIUS);
    xform.replace_shape_with_transformed(&mut us_data.turret_border_xformed_cache, &us_data.turret_border_vertexes);
    xform.replace_shape_with_transformed(&mut us_data.turret_inner_xformed_cache, &us_data.turret_inner_vertexes);
    let Shape::Polygon(ref turret_border_xformed) = us_data.turret_border_xformed_cache else {panic!()};
    let Shape::Polygon(ref turret_inner_xformed) = us_data.turret_inner_xformed_cache else {panic!()};
    let turret_border_dop = ctx.draw_ctx.do_thick_border(
        TURRET_BORDER_COLOR, 
        &turret_border_xformed.vertexes, 
        &turret_inner_xformed.vertexes);
    let turret_inner_dop = ctx.draw_ctx.do_tri_fan(us_data.turret_inner_color, &turret_inner_xformed.vertexes);
    let dop_group = ctx.draw_ctx.dop_group(Box::new([circles_dop, turret_border_dop, turret_inner_dop]));
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop_group);
}