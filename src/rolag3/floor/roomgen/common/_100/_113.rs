use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::regular_tri::small_rgb_circle::new_regtri_small_rgb_circle, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common113 contains nine RegtriSmallRgbCircles of N different colors
 */

pub fn get_gen_room_fn_common113(
    w: u32, 
    h: u32,
    num_colors: usize,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, num_colors)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32, 
    num_colors: usize,
) -> GenFloorRoomResponse {
    assert!(w >= 30, "w({}) is too low", w);
    assert!(h >= 30, "h({}) is too low", h);
    assert!(num_colors>=1 && num_colors<=3, "expected 1<=num_colors({})<=3", num_colors);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let damage_colors = rng.choose_multiple(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue], num_colors);
    for i in 2..=4 {
        for j in 2..=4 {
            let x = i as f64 * (w as f64 / 6.0);
            let y = j as f64 * (h as f64 / 6.0);
            let damage_color = rng.sample_slice_uniform(&damage_colors);
            let enemy = new_regtri_small_rgb_circle(nro_ctx, damage_color, x, y);
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