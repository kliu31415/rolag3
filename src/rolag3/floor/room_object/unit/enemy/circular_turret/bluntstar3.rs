/* Circular Turret BluntStar3 fires projectiles of a given color
*/

use std::{rc::Rc, cell::RefCell};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext}, standard_unit_common::RotateMove}, damage::DamageColor, projectile::projectile2::{Proj2Shape, Projectile2BuilderReq, Projectile2Builder}}, draw::{Color, DrawContext}, rofiz::rofiz_object::Transformation}, geometry::{shape::{Point, Shape, Vector}, star::get_blunt_star, util::get_inner_polygon}};

const CIRCLE_BORDER_RADIUS: f32 = 1.5;
const CIRCLE_INNER_RADIUS: f32 = 1.4;
const TURRET_INNER_RADIUS: f32 = 0.5;
const TURRET_OUTER_RADIUS: f32 = 1.2;
const TURRET_BORDER_THICKNESS: f32 = 0.1;
const PROJ_RADIUS: f32 = 0.2;
const PROJ_SHOOT_INTERVAL: f64 = 0.2;

struct CirTurBluntStar3 {
    damage_color: DamageColor,
    circle_inner_color: Color,
    turret_inner_color: Color,
    proj_origin_color: Color,
    proj_color: Color,
    turret_border_vertexes: Shape,
    turret_inner_vertexes: Shape,
    turret_border_xformed_cache: Shape,
    turret_inner_xformed_cache: Shape,
    proj_origin: Point,
    since_shot_proj: f64,
}

pub fn new_circular_turret_bluntstar3(ctx: &mut NewRoomObjectContext, x: f64, y: f64, color: DamageColor) -> StandardUnit1 {
    let turret_border_vertexes = get_blunt_star(3, TURRET_INNER_RADIUS, TURRET_OUTER_RADIUS, -std::f32::consts::FRAC_PI_3);
    let turret_inner_vertexes = get_inner_polygon(TURRET_BORDER_THICKNESS, &turret_border_vertexes);
    let xform = Transformation::new(x, y, 0.0);
    let circle_inner_color = match color {
        DamageColor::Red => Color::new(0.2, 0.1, 0.1, 1.0),
        DamageColor::Green => Color::new(0.1, 0.2, 0.1, 1.0),
        DamageColor::Blue => Color::new(0.1, 0.1, 0.2, 1.0),
        _ => panic!("can't create turret with color {:?}", color),
    };
    let turret_inner_color = match color {
        DamageColor::Red => Color::new(0.2, 0.01, 0.01, 1.0),
        DamageColor::Green => Color::new(0.01, 0.2, 0.01, 1.0),
        DamageColor::Blue => Color::new(0.01, 0.01, 0.2, 1.0),
        _ => panic!("can't create turret with color {:?}", color),
    };
    let proj_origin_color = match color {
        DamageColor::Red => Color::new(1.0, 0.01, 0.01, 1.0),
        DamageColor::Green => Color::new(0.01, 1.0, 0.01, 1.0),
        DamageColor::Blue => Color::new(0.01, 0.01, 1.0, 1.0),
        _ => panic!("can't create turret with color {:?}", color),
    };
    let proj_color = match color {
        DamageColor::Red => Color::new(5.0, 0.1, 0.1, 1.0),
        DamageColor::Green => Color::new(0.1, 1.5, 0.1, 1.0),
        DamageColor::Blue => Color::new(0.1, 0.1, 14.0, 1.0),
        _ => panic!("can't create turret with color {:?}", color),
    };
    let blunt_midpoint = Point::lerp(turret_border_vertexes[1], turret_border_vertexes[2], 0.5);
    let vec_to_proj_origin = (turret_border_vertexes[2] - blunt_midpoint).rotated(std::f32::consts::FRAC_PI_2);
    let proj_origin = &blunt_midpoint + vec_to_proj_origin;
    let us_data = CirTurBluntStar3 {
        damage_color: color,
        circle_inner_color,
        turret_inner_color,
        proj_origin_color,
        proj_color,
        turret_border_vertexes: Shape::of_polygon(turret_border_vertexes),
        turret_inner_vertexes: Shape::of_polygon(turret_inner_vertexes),
        turret_border_xformed_cache: Shape::default(),
        turret_inner_xformed_cache: Shape::default(),
        proj_origin,
        since_shot_proj: PROJ_SHOOT_INTERVAL,
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: color,
        collision_damage: 0.0,
        hp: 1.0, // dummy
        engine_power: 1.0, // dummy
        tire_traction: 1.0, // dummy
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, Shape::of_circle(Point::new(0.0, 0.0), CIRCLE_BORDER_RADIUS))
        .us_data(Box::new(us_data))
        .angular_power(1.0)
        .angular_traction(10.0)
        .damageable(false)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<CirTurBluntStar3>().unwrap();
    let mut response = Act1Response::new();
    let xform = ctx.act1_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate { atheta: 1.0 });
    us_data.since_shot_proj -= ctx.su_ctx.su_common.get_unit_tick_len();
    if us_data.since_shot_proj < 0.0 {
        let self_as_weak = ctx.act1_ctx.get_self_as_weak();
        let mut nfo_ctx = NewRoomObjectContext::from_act1_ctx(ctx.act1_ctx);
        for i in 0..3 {
            let angle = xform.dtheta + i as f64 * 2.0 / 3.0 * std::f64::consts::PI;
            let proj_speed = 12.0;
            let center = us_data.proj_origin.rotated(angle as f32);
            let shape = Proj2Shape::Circle {
                x: 0.0,
                y: 0.0,
                r: PROJ_RADIUS,
            };
            let proj_xform = Transformation::new(xform.dx + center.x as f64, xform.dy + center.y as f64, 0.0);
            let proj = Projectile2Builder::new(
                Projectile2BuilderReq {
                    team: Team::Enemy,
                    damage_color: us_data.damage_color,
                    damage: 3.0,
                    owner: self_as_weak.clone(),
                    velocity_x: proj_speed * f64::cos(angle),
                    velocity_y: proj_speed * f64::sin(angle),
                    xform: proj_xform,
                    shape,
                    color: us_data.proj_color,
                }
            ).build(&mut nfo_ctx);
            response.add_room_obj(Rc::new(RefCell::new(proj)));
        }
        us_data.since_shot_proj += PROJ_SHOOT_INTERVAL;
    }
    response
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<CirTurBluntStar3>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    let circles_dop = ctx.draw_ctx.do_concentric_circle(
        us_data.circle_inner_color,
        DrawContext::COLOR_NSU_BORDER, 
        Point::new(xform.dx as f32, xform.dy as f32), 
        CIRCLE_INNER_RADIUS, 
        CIRCLE_BORDER_RADIUS);
    xform.replace_shape_with_transformed(&mut us_data.turret_border_xformed_cache, &us_data.turret_border_vertexes);
    xform.replace_shape_with_transformed(&mut us_data.turret_inner_xformed_cache, &us_data.turret_inner_vertexes);
    let Shape::Polygon(ref turret_border_xformed) = us_data.turret_border_xformed_cache else {panic!()};
    let Shape::Polygon(ref turret_inner_xformed) = us_data.turret_inner_xformed_cache else {panic!()};
    let turret_border_dop = ctx.draw_ctx.do_thick_border(
        DrawContext::COLOR_NSU_BORDER, 
        &turret_border_xformed.vertexes, 
        &turret_inner_xformed.vertexes);
    let turret_inner_dop = ctx.draw_ctx.do_tri_fan(us_data.turret_inner_color, &turret_inner_xformed.vertexes);
    let mut dop_group = vec![circles_dop, turret_border_dop, turret_inner_dop];
    assert!(us_data.since_shot_proj >= 0.0);
    let shoot_delta_t = f64::min(us_data.since_shot_proj, f64::max(0.0, PROJ_SHOOT_INTERVAL - us_data.since_shot_proj));
    let lerp_t = f64::min(1.0, shoot_delta_t * 5.0 / PROJ_SHOOT_INTERVAL);
    let proj_inner_color = Color::lerp(us_data.proj_color, us_data.proj_origin_color, lerp_t as f32);
    for i in 0..3 {
        let center = us_data.proj_origin
            .rotated(xform.dtheta as f32 + i as f32 * 2.0 / 3.0 * std::f32::consts::PI)
            .translated(Vector::new(xform.dx as f32, xform.dy as f32));
        dop_group.push(ctx.draw_ctx.do_circle(proj_inner_color, center, PROJ_RADIUS));
    }
    let dop_group = ctx.draw_ctx.dop_group(dop_group.into());
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop_group);
}