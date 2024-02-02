use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::{regular_tri::small_rgb_circle_or_tri::{new_regtri_small_rgb_circle, new_regtri_small_rgb_tri}, small_square::rgb_circle_or_tri::{new_small_square_rgb_circle, new_small_square_rgb_tri}}, damage::DamageColor}, room::{RoomBuilder, RoomBuilderReq}};

/* Common113 contains many RegtriSmallRgbCircles/Tris of N different colors
 */

pub fn get_gen_room_fn_common113_tri_circle(
    w: u32, 
    h: u32,
    num_colors: usize,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, false, true, 3, 3, num_colors)
    })
}

pub fn get_gen_room_fn_common113_tri_tri(
    w: u32, 
    h: u32,
    num_colors: usize,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, false, false, 2, 3, num_colors)
    })
}

pub fn get_gen_room_fn_common113_square_circle(
    w: u32, 
    h: u32,
    num_colors: usize,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, true, true, 3, 3, num_colors)
    })
}

pub fn get_gen_room_fn_common113_square_tri(
    w: u32, 
    h: u32,
    num_colors: usize,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, true, false, 2, 4, num_colors)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32,
    is_border_square: bool,
    is_inner_circle: bool,
    ilen: usize,
    jlen: usize,
    num_colors: usize,
) -> GenFloorRoomResponse {
    assert!(w >= 30, "w({}) is too low", w);
    assert!(h >= 30, "h({}) is too low", h);
    assert!(ilen>=1&& ilen<=5, "expected 1<=ilen({})<=5", ilen);
    assert!(jlen>=1&& jlen<=5, "expected 1<=jlen({})<=5", jlen);
    assert!(num_colors>=1 && num_colors<=3, "expected 1<=num_colors({})<=3", num_colors);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    let damage_colors = rng.choose_multiple(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue], num_colors);
    for i in 2..=(1+ilen) {
        for j in 2..=(1+jlen) {
            let x = i as f64 * (w as f64 / ((4 + ilen) as f64));
            let y = j as f64 * (h as f64 / ((4 + jlen) as f64));
            let damage_color = rng.sample_slice_uniform(&damage_colors);
            let enemy = if is_border_square {
                if is_inner_circle {
                    new_small_square_rgb_circle(nro_ctx, damage_color, x, y)
                } else {
                    new_small_square_rgb_tri(nro_ctx, damage_color, x, y)
                }
            } else {
                if is_inner_circle {
                    new_regtri_small_rgb_circle(nro_ctx, damage_color, x, y)
                } else {
                    new_regtri_small_rgb_tri(nro_ctx, damage_color, x, y)
                }
            };
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    let enemy_coeff = if is_inner_circle {
        2.0
    } else {
        2.5
    };
    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.1 * f64::sqrt((w * h) as f64) + enemy_coeff * ((ilen * jlen) as f64),
                connection_candidates: all_borders_as_connection_candidates(w, h),
            },
        ),
    }
}