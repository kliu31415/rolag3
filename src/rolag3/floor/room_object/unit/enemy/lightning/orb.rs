use std::any::Any;

use crate::{rolag3::floor::{rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Transformation, Hitbox, RofizObjectMovement}}, room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, StandardUnit1, RofizObjType, SuAct1Context}, damage::DamageColor}}, geometry::shape::{Point, Shape}};

pub struct Orb {
    pub ro_ref: RofizObjectRef,
}

pub fn new_orb(ctx: &mut NewRoomObjectContext, x: f64, y: f64, radius: f32) -> StandardUnit1 {
    let mut builder = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color: DamageColor::Silver,
        collision_damage: 10.0,
        hp: 1.0, // dummy
        engine_power: 0.0, // dummy,
        tire_traction: 0.0, // dummy,
    });
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), radius);
    let hitbox = Hitbox::new(xform, shape);
    let room_obj_ref = builder.get_room_obj_metadata(ctx).get_ref();
    let ro_ref = ctx.add_spectral_unit(room_obj_ref, hitbox);
    let us_data = Orb { 
        ro_ref,
    };

    builder.slave_act1_fn(Box::new(orb_custom_act1))
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .us_data(Box::new(us_data))
        .damageable(false)
        .build(ctx)
}

fn orb_custom_act1(ctx: &mut SuAct1Context, _: &mut Act1Response, input: &dyn Any, _output: &mut dyn Any) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Orb>().unwrap();
    let orb_movement = *input.downcast_ref::<(f64, f64)>().unwrap();
    let xform = Transformation::new(orb_movement.0, orb_movement.1, 0.0);
    ctx.act1_ctx.get_rofiz().move_object(&us_data.ro_ref, RofizObjectMovement::SetXform(xform));
}