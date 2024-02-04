use std::{any::Any, collections::VecDeque};

use crate::{rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::standard_unit1::{StandardUnit1Builder, StandardUnit1BuilderReq, StandardUnit1, RofizObjType, SuAct1Context}, damage::DamageColor}, rofiz::{rofiz_object::{Transformation, Hitbox, RofizObjectMovement}, rofiz_state::RofizObjectRef}}, geometry::shape::{Shape, Point}};


struct SwordSlash {
    // all rofo refs are kept under a single RoomObject as an optimization. RofizObjects belonging to the same room
    // objects won't have their shapes checked for collisions in Rofiz, which is a huge speed improvement.
    rofo_refs: VecDeque<RofizObjectRef>,
}

pub fn new_sword_slash(
    ctx: &mut NewRoomObjectContext, 
    team: Team,
    dps: f64,
) -> StandardUnit1 {
    let us_data = SwordSlash { 
        rofo_refs: VecDeque::new(),
    };
    
    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team,
        damage_color: DamageColor::Blue,
        hp: 0.0, // dummy
        engine_power: 0.0, // dummy,
        tire_traction: 0.0, // dummy,
    }).slave_act1_fn(Box::new(slave_act1))
        .collision_damage(dps)
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .us_data(Box::new(us_data))
        .damageable(false)
        .blocks_room_clear(false)
        .build(ctx)
}

fn slave_act1(ctx: &mut SuAct1Context, _: &mut Act1Response, input: &dyn Any, _output: &mut dyn Any) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<SwordSlash>().unwrap();
    let (x, y, del, add) = input.downcast_ref::<(f64, f64, usize, Vec<[Point; 3]>)>().unwrap();
    let xform = Transformation::new(*x, *y, 0.0);

    for _ in 0..*del {
        us_data.rofo_refs.pop_front();
    }
    for vertexes in add {
        let xform = Transformation::new(*x, *y, 0.0);
        let shape = Shape::of_polygon(Box::new(*vertexes));
        let hitbox = Hitbox::new(xform, shape);
        // make slashes a spectral unit so they interact with projectiles
        let rofo_ref = ctx.act1_ctx.get_rofiz().add_spectral_unit(ctx.su_ctx.md.get_ref(), hitbox);
        us_data.rofo_refs.push_back(rofo_ref);
    }
    for r in us_data.rofo_refs.iter() {
        ctx.act1_ctx.get_rofiz().move_object(r, RofizObjectMovement::SetXform(xform));
    }
}