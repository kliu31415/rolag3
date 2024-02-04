use std::any::Any;

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, StandardUnit1, RofizObjType, SuAct1Context}, damage::DamageColor}, rofiz::{rofiz_object::{Transformation, Hitbox, RofizObjectMovement}, rofiz_state::RofizObjectRef}}, geometry::shape::{Shape, Point}};


struct SwordSlash {
    rofo_ref: RofizObjectRef,
}

pub fn new_sword_slash(
    ctx: &mut NewRoomObjectContext, 
    team: Team,
    dps: f64,
    x: f64, 
    y: f64, 
    vertexes: Box<[Point]>,
) -> StandardUnit1 {
    let mut builder = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team,
        damage_color: DamageColor::Blue,
        hp: 0.0, // dummy
        engine_power: 0.0, // dummy,
        tire_traction: 0.0, // dummy,
    });
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_polygon(vertexes);
    let hitbox = Hitbox::new(xform, shape);
    let room_obj_ref = builder.get_room_obj_metadata(ctx).get_ref();
    // make slashes a spectral unit so they interact with projectiles
    let rofo_ref = ctx.add_spectral_unit(room_obj_ref, hitbox);
    let us_data = SwordSlash { 
        rofo_ref,
    };

    builder.slave_act1_fn(Box::new(slave_act1))
        .collision_damage(dps)
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .us_data(Box::new(us_data))
        .damageable(false)
        .blocks_room_clear(false)
        .build(ctx)
}

fn slave_act1(ctx: &mut SuAct1Context, _: &mut Act1Response, input: &dyn Any, _output: &mut dyn Any) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SwordSlash>().unwrap();
    let (x, y) = *input.downcast_ref::<(f64, f64)>().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    ctx.act1_ctx.get_rofiz().move_object(&us_data.rofo_ref, RofizObjectMovement::SetXform(xform));
}