

use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{square_room::init_basic_square_room, connection_candidates::all_borders_as_connection_candidates}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::hexagon::rgb2_circle::new_hexagon_rgb2_circle, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common110 contains several HexagonRgb2_Circles
 */

pub fn get_gen_room_fn_common110a(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, 2, 1)
    })
}

pub fn get_gen_room_fn_common110b(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, 2, 2)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, w: u32, h: u32, enemy_maxi: u32, enemy_maxj: u32) -> GenFloorRoomResponse {
    assert!(w >= 30, "w({}) is too low", w);
    assert!(h >= 30, "h({}) is too low", h);
    assert!((0..=3).contains(&enemy_maxi), "enemy_maxi outside of expected range");
    assert!((0..=3).contains(&enemy_maxj), "enemy_maxj outside of expected range");
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    for i in 0..enemy_maxi {
        for j in 0..enemy_maxj {
            let x = w as f64 * (i as f64 + 1.0) / (enemy_maxi as f64 + 1.0);
            let y = h as f64 * (j as f64 + 1.0) / (enemy_maxj as f64 + 1.0);
            let damage_color = rng.sample_slice_uniform(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue]);
            let enemy = new_hexagon_rgb2_circle(nro_ctx, damage_color, x, y);
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.1 * f64::sqrt((w * h) as f64) + 20.0,
                connection_candidates: all_borders_as_connection_candidates(w, h),
            },
        ),
    }
}