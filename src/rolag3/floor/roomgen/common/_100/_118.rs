use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room_middle_walled}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::small_square::rgb_circle_or_tri::{new_small_square_rgb_circle, new_small_square_rgb_tri}, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common118 contains a wall block in the middle, along with many SmallSquareRgbCircles/Tris
 */

pub fn get_gen_room_fn_common118a(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, 0.0)
    })
}

pub fn get_gen_room_fn_common118b(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, 0.5)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32,
    tri_prob: f64,
) -> GenFloorRoomResponse {
    assert!(w >= 30, "w({}) is too low", w);
    assert!(h >= 30, "h({}) is too low", h);
    assert!(w % 5 == 0, "w({}) not divisible by 5", w);
    assert!(h % 5 == 0, "h({}) not divisible by 5", h);
    let mut rng1 = ctx.rng.spawn_child();
    let mut rng2 = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room_middle_walled(ctx, w, h, w / 5, h / 5);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let colors_rgb = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
    let mut colors = (0..8).map(|_| rng1.sample_slice_uniform(&colors_rgb));
    for i in [0.3, 0.5, 0.7] {
        for j in [0.3, 0.5, 0.7] {
            if i == 0.5 && j == 0.5 {
                continue;
            }
            let x = w as f64 * i;
            let y = h as f64 * j;
            let enemy = if rng2.gen_bernoulli(tri_prob) {
                new_small_square_rgb_tri(nro_ctx, colors.next().unwrap(), x, y)
            } else {
                new_small_square_rgb_circle(nro_ctx, colors.next().unwrap(), x, y)
            };
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
                ttc: 0.1 * f64::sqrt((w * h) as f64) + 12.0,
                connection_candidates: all_borders_as_connection_candidates(w, h),
            },
        ),
    }
}