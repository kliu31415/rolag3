use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::{init_basic_square_room, init_basic_square_room_middle_walled}}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::small_octagon::rgb_circle::new_small_octagon_rgb_circle, damage::DamageColor, tiles::black_hole::new_black_hole}, room::{RoomBuilder, RoomBuilderReq}};

/* Common116 contains many SmallOctagonRgbCircles
 */

pub fn get_gen_room_fn_common116a(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let mut colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        ctx.rng.shuffle(&mut colors);
        let oct_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors[1..]));
        make_room(ctx, w, h, false, false, oct_colors)
    })
}

pub fn get_gen_room_fn_common116b(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        let oct_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors));
        make_room(ctx, w, h, false, false, oct_colors)
    })
}

pub fn get_gen_room_fn_common116c(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        let oct_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors));
        make_room(ctx, w, h, true, false, oct_colors)
    })
}

pub fn get_gen_room_fn_common116d(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        let colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        let oct_colors = std::array::from_fn(|_| ctx.rng.sample_slice_uniform(&colors));
        make_room(ctx, w, h, false, true, oct_colors)
    })
}


fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32,
    black_hole: bool,
    middle_wall: bool,
    oct_colors: [DamageColor; 4],
) -> GenFloorRoomResponse {
    assert!(w >= 25, "w({}) is too low", w);
    assert!(h >= 25, "h({}) is too low", h);
    let (mut rofiz, mut room_objects) = if middle_wall {
        assert!(w % 5 == 0, "expected w({}) to be divisible by 5", w);
        assert!(h % 5 == 0, "expected h({}) to be divisible by 5", h);
        init_basic_square_room_middle_walled(ctx, w, h, w / 5, h / 5)
    } else {
        init_basic_square_room(ctx, w, h)
    };
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let mut oct_colors = oct_colors.into_iter();
    for i in [0.3, 0.7] {
        for j in [0.3, 0.7] {
            let x = w as f64 * i;
            let y = h as f64 * j;
            let enemy = new_small_octagon_rgb_circle(nro_ctx, oct_colors.next().unwrap(), x, y);
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    if black_hole {
        let colors = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
        let color = nro_ctx.get_rng().sample_slice_uniform(&colors);
        let bh = new_black_hole(nro_ctx, Some(color), w as f64 / 2.0, h as f64 / 2.0);
        room_objects.add(Rc::new(RefCell::new(bh)));
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