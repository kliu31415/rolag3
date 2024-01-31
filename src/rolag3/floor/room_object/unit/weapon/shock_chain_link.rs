use std::any::Any;

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, StandardUnit1, RofizObjType, SuAct1Context}, damage::DamageColor}, rofiz::{rofiz_object::{Transformation, Hitbox}, rofiz_state::RofizObjectRef}}, geometry::shape::{Shape, Point}};


struct ShockChainLink {
    rofo_ref: RofizObjectRef,
    radius: f32,
}

pub fn new_shock_chain_link(
    ctx: &mut NewRoomObjectContext, 
    team: Team,
    x: f64, 
    y: f64, 
    dps: f64,
    radius: f32,
) -> StandardUnit1 {
    let mut builder = StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team,
        damage_color: DamageColor::Green,
        hp: 0.0, // dummy
        engine_power: 0.0, // dummy,
        tire_traction: 0.0, // dummy,
    });
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), radius);
    let hitbox = Hitbox::new(xform, shape);
    let room_obj_ref = builder.get_room_obj_metadata(ctx).get_ref();
    let rofo_ref = ctx.add_basic_projectile(room_obj_ref, hitbox);
    let us_data = ShockChainLink { 
        rofo_ref,
        radius
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
    let us_data = ctx.su_ctx.us_data.downcast_mut::<ShockChainLink>().unwrap();
    let (x, y) = *input.downcast_ref::<(f64, f64)>().unwrap();
    let xform = Transformation::new(x, y, 0.0);
    let shape = Shape::of_circle(Point::new(0.0, 0.0), us_data.radius);
    let hitbox = Hitbox::new(xform, shape);
    // recreate the Rofiz object every tick. Otherwise, if the player moves rooms from R1 to R2, the rofo_ref will
    // still reference R1's Rofiz, which causes bugs.
    us_data.rofo_ref = ctx.act1_ctx.get_rofiz().add_basic_projectile(ctx.su_ctx.md.get_ref(), hitbox);
}