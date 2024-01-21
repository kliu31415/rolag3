use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::standard_unit1::{StandardUnit1, StandardUnit1Builder, StandardUnit1BuilderReq, SuAct1Context, SuDrawContext, RofizObjType}, damage::DamageColor}, draw::{Color, DrawContext}, rofiz::{rofiz_object::{Transformation, RofizObjectMovement, Hitbox}, rofiz_state::RofizObjectRef}}, geometry::shape::{Shape, Rect, Point}};

/* RotatingLaserBasic is a single laser that rotates at a slow speed
*/

const CIRCULAR_BASE_BORDER_COLOR: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const CIRCULAR_BASE_BORDER_R: f32 = 0.5;
const CIRCULAR_BASE_INNER_R: f32 = 0.4;
const LASER_WIDTH: f32 = 0.15;
const LASER_LENGTH: f32 = 5.0;

struct RotatingLaserBasic {
    laser_draw_color: Color,
    circular_base_inner_draw_color: Color,
    rect: Shape,
    xformed_rect_cache: Shape,
    angular_speed: f64,

    rotating_laser_ro_ref: RofizObjectRef,
}

pub fn new_rotating_laser(
    ctx: &mut NewRoomObjectContext, 
    xform: Transformation,
    color: DamageColor,
    angular_speed: f64,
) -> StandardUnit1 {
    let draw_color = match color {
        DamageColor::Red => Color::new(5.0, 0.1, 0.1, 1.0),
        DamageColor::Green => Color::new(0.1, 1.5, 0.1, 1.0),
        DamageColor::Blue => Color::new(0.1, 0.1, 14.0, 1.0),
        _ => panic!("can't create rotating laser with color {:?}", color),
    };
    let circular_base_inner_draw_color = match color {
        DamageColor::Red => Color::new(0.5, 0.01, 0.01, 1.0),
        DamageColor::Green => Color::new(0.01, 0.5, 0.01, 1.0),
        DamageColor::Blue => Color::new(0.01, 0.01, 0.5, 1.0),
        _ => panic!("can't create rotating laser with color {:?}", color),
    };
    let rect = Shape::of_rect(Rect::new(0.0, -LASER_WIDTH / 2.0, LASER_LENGTH, LASER_WIDTH));

    let mut builder = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: color,
        hp: 1.0, // dummy
        engine_power: 1.0, // dummy
        tire_traction: 1.0, // dummy
    });
    let md = builder.get_room_obj_metadata(ctx);
    let rotating_laser_ro_ref = ctx.add_basic_projectile(md.get_ref(), Hitbox::new(xform, rect.clone()));

    let us_data = RotatingLaserBasic {
        laser_draw_color: draw_color,
        circular_base_inner_draw_color,
        rect: rect.clone(),
        xformed_rect_cache: Shape::default(),
        angular_speed,
        rotating_laser_ro_ref,
    };

    builder.act1_fn(Box::new(act1))
        .draw_fn(Box::new(draw))
        .hitbox(xform, Shape::of_circle(Point::new(0.0, 0.0), CIRCULAR_BASE_BORDER_R))
        .us_data(Box::new(us_data))
        .damageable(false)
        .blocks_room_clear(false)
        .add_secondary_hitbox(xform, rect.clone(), RofizObjType::BasicProjectile)
        .build(ctx)
}

fn act1(ctx: &mut SuAct1Context) -> Act1Response {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RotatingLaserBasic>().unwrap();
    let move_xform = Transformation::new(0.0, 0.0, ctx.su_ctx.su_common.get_unit_tick_len() * us_data.angular_speed);
    ctx.act1_ctx.get_rofiz().move_object(&us_data.rotating_laser_ro_ref, RofizObjectMovement::Move(move_xform));
    Act1Response::new()
}

fn draw(ctx: &mut SuDrawContext) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<RotatingLaserBasic>().unwrap();
    let xform = ctx.draw_ctx.get_rofiz().get_movable_object_xform(&us_data.rotating_laser_ro_ref);
    xform.replace_shape_with_transformed(&mut us_data.xformed_rect_cache, &us_data.rect);
    let Shape::Polygon(xformed_rect) = &us_data.xformed_rect_cache else {panic!()};
    let circular_base = ctx.draw_ctx.do_concentric_circle(
        us_data.circular_base_inner_draw_color, 
        CIRCULAR_BASE_BORDER_COLOR,
        Point::new(xform.dx as f32, xform.dy as f32),
        CIRCULAR_BASE_INNER_R,
        CIRCULAR_BASE_BORDER_R);
    let main_laser = ctx.draw_ctx.do_quad_fan(us_data.laser_draw_color, (*xformed_rect.vertexes).try_into().unwrap());
    let dop_group = ctx.draw_ctx.dop_group(Box::new([circular_base, main_laser]));
    ctx.draw_ctx.add_draw_op(DrawContext::Z_UNIT, dop_group);
}