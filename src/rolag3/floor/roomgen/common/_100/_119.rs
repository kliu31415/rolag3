use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room_middle_walled}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::{small_octagon::rgb_circle::new_small_octagon_rgb_circle, hexagon::rgb2_circle::new_hexagon_rgb2_circle, rotating_laser::laser::new_rotating_laser}, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}, rofiz::rofiz_object::Transformation};

/* Common119 contains a wall block in the middle, along with a laser and some HexagonRgb2Circles/SmallOctagonRgbCircles
 */

pub fn get_gen_room_fn_common119a(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, false, false)
    })
}

pub fn get_gen_room_fn_common119b(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, false, true)
    })
}

pub fn get_gen_room_fn_common119c(
    w: u32, 
    h: u32,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, true, true)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32,
    top_hex: bool,
    bot_hex: bool,
) -> GenFloorRoomResponse {
    assert!(w >= 30, "w({}) is too low", w);
    assert!(h >= 30, "h({}) is too low", h);
    assert!(w % 4 == 0, "w({}) not divisible by 4", w);
    assert!(h % 2 == 0, "h({}) not divisible by 2", h);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room_middle_walled(ctx, w, h, w / 4, 2);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let colors_rgb = [DamageColor::Red, DamageColor::Green, DamageColor::Blue];
    let mut colors = (0..4).map(|_| rng.sample_slice_uniform(&colors_rgb));
    for (_, xf) in [0.4, 0.6].into_iter().enumerate() {
        for (j, yf) in [0.4, 0.6].into_iter().enumerate() {
            let x = w as f64 * xf;
            let y = h as f64 * yf;
            let hex = if j == 0 {
                top_hex
            } else {
                bot_hex
            };
            let enemy = if hex {
                new_hexagon_rgb2_circle(nro_ctx, colors.next().unwrap(), x, y)
            } else {
                new_small_octagon_rgb_circle(nro_ctx, colors.next().unwrap(), x, y)
            };
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    for dy in [-2.0, 2.0] {
        let laser = new_rotating_laser(
            nro_ctx, 
            Transformation::new(w as f64 / 2.0, h as f64 / 2.0 - dy, rng.gen_f64_range(0.0 .. std::f64::consts::PI)), 
            rng.sample_slice_uniform(&colors_rgb), 
            0.5 * f64::min(w as f64, h as f64) - 7.0, 
            rng.gen_f64_range(0.5 .. 0.7),
        );
        room_objects.add(Rc::new(RefCell::new(laser)));
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