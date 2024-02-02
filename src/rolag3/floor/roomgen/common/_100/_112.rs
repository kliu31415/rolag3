use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::small_square::rgb::new_small_square_rgb, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common112 contains nine SmallSquareRgbs
 */

pub fn get_gen_room_fn_common112(
    w: u32, 
    h: u32, 
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32, 
) -> GenFloorRoomResponse {
    assert!(w >= 20, "w({}) is too low", w);
    assert!(h >= 20, "h({}) is too low", h);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    for i in 2..=4 {
        for j in 2..=4 {
            // perturb x and y by up to 0.1 to ensure no x and y coordinates is exactly the same across enemies.
            // This prevents enemies from sliding over each other
            let x = i as f64 * (w as f64 / 6.0) + rng.gen_f64_range(-0.1 .. 0.1);
            let y = j as f64 * (h as f64 / 6.0) + rng.gen_f64_range(-0.1 .. 0.1);
            let damage_color = rng.sample_slice_uniform(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue]);
            let enemy = new_small_square_rgb(nro_ctx, damage_color, x, y);
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
                ttc: 0.1 * f64::sqrt((w * h) as f64) + 15.0,
                connection_candidates: all_borders_as_connection_candidates(w, h),
            },
        ),
    }
}