use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::big_circle::rgb_diamond::new_big_circle_rgb_diamond, damage::DamageColor}, roomgen::util::{square_room::init_basic_square_room, connection_candidates::all_borders_as_connection_candidates}, room::{RoomBuilder, RoomBuilderReq}};

/* Common102 contains several BigCircleDiamonds (units that spawn diamonds).
 */

pub fn get_gen_room_fn_common102a() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let mut enemy_info = Vec::new();
        for i in 0..2 {
            for j in 0..2 {
                let x = (7 + 6*i) as f64;
                let y = (7 + 6*j) as f64;
                let damage_color = ctx.rng.sample_slice_uniform(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue]);
                enemy_info.push((x, y, damage_color))
            }
        }
        make_room(ctx, 20, 20, &enemy_info)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, 
    w: usize, 
    h: usize,
    enemy_info: &[(f64, f64, DamageColor)],
) -> GenFloorRoomResponse {
    assert!(w >= 15, "w({}) is too low", w);
    assert!(h >= 15, "h({}) is too low", h);
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    for (x, y, damage_color) in enemy_info {
        let enemy = new_big_circle_rgb_diamond(nro_ctx, *damage_color, *x, *y);
        room_objects.add(Rc::new(RefCell::new(enemy)));
    }

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w as u32,
                height: h as u32,
                room_objects,
                rofiz,
                ttc: 0.2 * f64::sqrt((w*h) as f64) + 10.0 * enemy_info.len() as f64,
                connection_candidates: all_borders_as_connection_candidates(w as u32, h as u32),
            },
        ),
    }
}