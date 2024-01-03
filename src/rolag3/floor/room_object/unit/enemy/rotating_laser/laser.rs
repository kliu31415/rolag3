use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::{standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, RofizObjType}, standard_unit_common::RotateMove}, damage::DamageColor}, draw::{Color, DrawContext}, rofiz::rofiz_object::Transformation}, geometry::shape::{Shape, Rect}};

/* RotatingLaserBasic is a single laser that rotates at a slow speed
*/

const LASER_WIDTH: f32 = 0.15;
const LASER_LENGTH: f32 = 5.0;

struct RotatingLaserBasic {
    rotate_dir: f64,
    draw_color: Color,
    rect: Shape,
    xformed_rect_cache: Shape,
}

pub fn new_rotating_laser(
    ctx: &mut NewRoomObjectContext, 
    xform: Transformation,
    color: DamageColor,
    angular_power: f64,
) -> StandardUnit1 {
    let draw_color = match color {
        DamageColor::Red => Color::new(5.0, 0.1, 0.1, 1.0),
        DamageColor::Green => Color::new(0.1, 1.5, 0.1, 1.0),
        DamageColor::Blue => Color::new(0.1, 0.1, 14.0, 1.0),
        _ => panic!("can't create rotating laser with color {:?}", color),
    };
    let (angular_power, rotate_dir) = if angular_power < 0.0 {
        (-angular_power, -1.0)
    } else {
        (angular_power, 1.0)
    };
    let rect = Shape::of_rect(Rect::new(0.0, -LASER_WIDTH / 2.0, LASER_LENGTH, LASER_WIDTH));
    let us_data = RotatingLaserBasic {
        rotate_dir,
        draw_color,
        rect: rect.clone(),
        xformed_rect_cache: Shape::default(),
    };

    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: color,
        hp: 1.0, // dummy
        engine_power: 1.0, // dummy
        tire_traction: 1.0, // dummy
    }).act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, rect)
        .us_data(Box::new(us_data))
        .angular_power(angular_power)
        .angular_traction(10.0)
        .damageable(false)
        .rofiz_obj_type(RofizObjType::BasicProjectile)
        .is_spectral(true)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RotatingLaserBasic>().unwrap();
    ctx.su_ctx.su_common.set_rotate_move(RotateMove::Accelerate { atheta: us_data.rotate_dir });
    Act1Response::new()
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RotatingLaserBasic>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(ctx.su_ctx.su_common.get_ro_ref());
    xform.replace_shape_with_transformed(&mut us_data.xformed_rect_cache, &us_data.rect);
    let Shape::Polygon(xformed_rect) = &us_data.xformed_rect_cache else {panic!()};
    let main_laser = ctx.draw_ctx.do_quad_fan(us_data.draw_color, (*xformed_rect.vertexes).try_into().unwrap());
    let dop_group = ctx.draw_ctx.dop_group(Box::new([main_laser]));
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop_group);
}