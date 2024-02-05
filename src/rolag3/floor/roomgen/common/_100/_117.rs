use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room_middle_walled}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::hexagon::rgb2_circle::new_hexagon_rgb2_circle, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common117 contains a wall block in the middle, along with many HexagonRgb2Circles
 */

pub fn get_gen_room_fn_common117a(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let mut colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        ctx.rng.shuffle(&mut colors);
        let hex_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors[1..]));
        make_room(ctx, w, h, hex_colors)
    })
}

pub fn get_gen_room_fn_common117b(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let mut colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        ctx.rng.shuffle(&mut colors);
        let hex_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors));
        make_room(ctx, w, h, hex_colors)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32,
    oct_colors: [DamageColor; 4],
) -> GenFloorRoomResponse {
    assert!(w >= 30, "w({}) is too low", w);
    assert!(h >= 30, "h({}) is too low", h);
    assert!(w % 5 == 0, "w({}) not divisible by 5", w);
    assert!(h % 5 == 0, "h({}) not divisible by 5", h);
    let (mut rofiz, mut room_objects) = init_basic_square_room_middle_walled(ctx, w, h, w / 5, h / 5);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let mut hex_colors = oct_colors.into_iter();
    for i in [0.3, 0.7] {
        for j in [0.3, 0.7] {
            let x = w as f64 * i;
            let y = h as f64 * j;
            let enemy = new_hexagon_rgb2_circle(nro_ctx, hex_colors.next().unwrap(), x, y);
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