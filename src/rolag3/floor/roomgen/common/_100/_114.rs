use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::{hexagon::rgb2_circle::new_hexagon_rgb2_circle, square::rgb_square2::new_square_rgb_square2}, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common114 contains two HexagonRgb2Circles and two SquareRgbSquare2s
 */

pub fn get_gen_room_fn_common114(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        let hexagon_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors));
        let square_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors));
        make_room(ctx, w, h, hexagon_colors, square_colors)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32,
    hexagon_colors: [DamageColor; 2],
    square_colors: [DamageColor; 2],
) -> GenFloorRoomResponse {
    assert!(w >= 25, "w({}) is too low", w);
    assert!(h >= 25, "h({}) is too low", h);
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    for (i, colors) in [hexagon_colors, square_colors].into_iter().enumerate() {
        for (j, color) in colors.into_iter().enumerate() {
            let x = w as f64 * (i + 1) as f64 / 3.0;
            let y = h as f64 * (j + 1) as f64 / 3.0;
            let enemy = if i % 2 == 0 {
                Rc::new(RefCell::new(new_hexagon_rgb2_circle(nro_ctx, color, x, y)))
            } else {
                Rc::new(RefCell::new(new_square_rgb_square2(nro_ctx, color, x, y)))
            };
            room_objects.add(enemy);
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