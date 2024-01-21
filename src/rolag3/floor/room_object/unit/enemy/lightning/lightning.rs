use std::any::Any;

use crate::{rolag3::floor::{rofiz::{rofiz_state::RofizObjectRef, rofiz_object::{Transformation, RofizObjectMovement, Hitbox}}, draw::Color, room_object::{room_object_def::{NewRoomObjectContext, Team, Act1Response}, unit::standard_unit1::{StandardUnit1BuilderReq, StandardUnit1, StandardUnit1Builder, RofizObjType, SuAct1Context, Su1Data}, damage::DamageColor}}, geometry::shape::{Point, Shape}};

use super::chunked_brownian_bridge::ChunkedBrownianBridge;

// Code for lightning that arcs between orbs
struct Lightning {
    draw_color: Color,
    ro_refs: Vec<RofizObjectRef>,
    cbb_quad_cache: Vec<[Point; 4]>,
    bridge1: ChunkedBrownianBridge,
    bridge2: ChunkedBrownianBridge,
    lerped_bridge: ChunkedBrownianBridge,
    lerp_t: f64,
    cur_cbb_per_s: f64,

    cbb_per_s_mean: f64,
    cbb_per_s_sd: f64,
    cbb_per_s_min: f64,
    quad_thickness: f32,
}

pub fn new_lightning(
    ctx: &mut NewRoomObjectContext, 
    damage_color: DamageColor,
    draw_color: Color,
    cbb_per_s_mean: f64,
    cbb_per_s_sd: f64,
    cbb_per_s_min: f64,
    quad_thickness: f32,
) -> StandardUnit1 {
    // note that the first CBBs are straight lines
    let bridge1 = ChunkedBrownianBridge::new(ctx.get_rng(), 1, 2.0, 1.0);
    let bridge2 = ChunkedBrownianBridge::new(ctx.get_rng(), 1, 2.0, 1.0);
    let lerped_bridge = ChunkedBrownianBridge::lerp(&bridge1, &bridge2, 1.0);
    let lightning = Lightning {
        draw_color,
        ro_refs: Vec::new(),
        cbb_quad_cache: Vec::new(),
        bridge1,
        bridge2,
        lerped_bridge, 
        lerp_t: 1.0,
        cur_cbb_per_s: cbb_per_s_mean,
        cbb_per_s_mean,
        cbb_per_s_sd,
        cbb_per_s_min,
        quad_thickness,
    };
    StandardUnit1Builder::new(StandardUnit1BuilderReq {
        team: Team::Enemy,
        damage_color,
        hp: 1.0, // dummy
        engine_power: 0.0, // dummy
        tire_traction: 0.0, // dummy
    }).slave_act1_fn(Box::new(lightning_custom_act1))
        .add_custom_fn(Box::new(lightning_custom_get_cbb))
        .rofiz_obj_type(RofizObjType::SpectralUnit)
        .us_data(Box::new(lightning))
        .damageable(false)
        .blocks_room_clear(false)
        .build(ctx)
}

fn lightning_custom_act1(ctx: &mut SuAct1Context, _: &mut Act1Response, input: &dyn Any, _output: &mut dyn Any) {
    let us_data = ctx.su_ctx.us_data.downcast_mut::<Lightning>().unwrap();
    let (orb1_xform, orb2_xform, y_sd) = input.downcast_ref::<(Transformation, Transformation, f64)>().unwrap();
    let start = Point::new(orb1_xform.dx as f32, orb1_xform.dy as f32);
    let end = Point::new(orb2_xform.dx as f32, orb2_xform.dy as f32);
    let tick_len = ctx.act1_ctx.get_tick_length();
    us_data.lerp_t += tick_len * us_data.cur_cbb_per_s;
    if us_data.lerp_t >= 1.0 {
        std::mem::swap(&mut us_data.bridge1, &mut us_data.bridge2);
        let dist = (end - start).norm();
        let num_chunks = usize::clamp((dist / 2.0) as usize, 1, 20);
        us_data.bridge2 = ChunkedBrownianBridge::new(ctx.act1_ctx.get_rng(), num_chunks, 3.0, *y_sd);
        us_data.lerp_t = 0.0;
        us_data.cur_cbb_per_s = ctx.act1_ctx.get_rng().gen_normal(us_data.cbb_per_s_mean, us_data.cbb_per_s_sd);
        us_data.cur_cbb_per_s = f64::max(us_data.cbb_per_s_min, us_data.cur_cbb_per_s);
    }
    us_data.lerped_bridge = ChunkedBrownianBridge::lerp(&us_data.bridge1, &us_data.bridge2, us_data.lerp_t);
    us_data.lerped_bridge.to_quads(&mut us_data.cbb_quad_cache, us_data.quad_thickness, start, end);
    if us_data.ro_refs.len() > us_data.cbb_quad_cache.len() {
        us_data.ro_refs.truncate(us_data.cbb_quad_cache.len());
    }
    for (i, q) in us_data.cbb_quad_cache.drain(..).enumerate() {
        // shift the quad so that its non-transformed hitbox is around the origin. This isn't useful now but may be
        // later if code is added that assumes all hitboxes are around the origin, and the xform represents the
        // rough coordinates of the hitbox
        let shifted_q = q.map(|p| Point::new(p.x - q[0].x, p.y - q[0].y));
        let xform = Transformation::new(q[0].x as f64, q[0].y as f64, 0.0);
        // TODO: optimize this allocation
        if i < us_data.ro_refs.len() {
            let mut stolen_hitbox = ctx.act1_ctx.get_rofiz().steal_movable_object_hitbox(&us_data.ro_refs[i]);
            stolen_hitbox.transformation = xform;
            stolen_hitbox.shape.replace_with_polygon(&shifted_q);
            let movement = RofizObjectMovement::NewHitbox(stolen_hitbox);
            ctx.act1_ctx.get_rofiz().move_object(&us_data.ro_refs[i], movement);
        } else {
            let shape = Shape::of_polygon(Box::new(shifted_q));
            us_data.ro_refs.push(ctx.act1_ctx.get_rofiz().add_basic_projectile(ctx.su_ctx.md.get_ref(), Hitbox::new(xform, shape)));
        }
    }
}

fn lightning_custom_get_cbb(data: &mut Su1Data, _input: &dyn Any, output: &mut dyn Any) {
    let output_pair = output.downcast_mut::<(Option<ChunkedBrownianBridge>, Option<Color>)>().unwrap();
    let us_data = data.us_data.downcast_ref::<Lightning>().unwrap();
    *output_pair = (Some(us_data.lerped_bridge.clone()), Some(us_data.draw_color))
}
