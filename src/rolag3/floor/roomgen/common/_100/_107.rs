use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{square_room::init_basic_square_room, logfp::new_logfp_v1, connection_candidates::some_borders_as_connection_candidates}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::square::rgb_star4or8::{new_square_rgb_star4, new_square_rgb_star8}, damage::DamageColor, tiles::ice_tile::new_ice_tile}, room::{RoomBuilder, RoomBuilderReq}};

/* Common107 contains four SquareRgbStar4_8, a lightning orb group, as well as many ice tiles
 */

pub fn get_gen_room_fn_common107a() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, false, 30, 30, 30)
    })
}

pub fn get_gen_room_fn_common107b() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, true, 30, 30, 30)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, star8: bool, w: u32, h: u32, num_ice_tiles: usize) -> GenFloorRoomResponse {
    assert!(w >= 15, "w({}) is too low", w);
    assert!(h >= 15, "h({}) is too low", h);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);


    for i in 0..2 {
        for j in 0..2 {
            // perturb x and y by up to 0.1 to ensure no x and y coordinates is exactly the same across enemies.
            // This prevents enemies from sliding over each other
            let x = (i+1) as f64 * (w as f64 / 3.0) + rng.gen_f64_range(-0.1 .. 0.1);
            let y = (j+1) as f64 * (h as f64 / 3.0) + rng.gen_f64_range(-0.1 .. 0.1);
            let damage_color = rng.sample_slice_uniform(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue]);
            let enemy = if star8 {
                new_square_rgb_star8(nro_ctx, damage_color, x, y)
            } else {
                new_square_rgb_star4(nro_ctx, damage_color, x, y)
            };
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    let (logfp, connection_borders) = new_logfp_v1(nro_ctx, w, h);
    logfp.into_vec().into_iter().for_each(|x| room_objects.add(x));

    let mut candidates = (4..(w-4)).flat_map(|x| (4..(h-4)).map(move |y| (x, y))).collect::<Box<_>>();
    assert!(num_ice_tiles <= candidates.len(), "expected num_ice_tiles({}) < candidates.len({})", num_ice_tiles, candidates.len());
    rng.shuffle(&mut candidates);

    for i in 0..num_ice_tiles {
        let enemy = new_ice_tile(nro_ctx, candidates[i].0, candidates[i].1);
        room_objects.add(Rc::new(RefCell::new(enemy)));
    }

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.1 * f64::sqrt((w * h) as f64) + 20.0,
                connection_candidates: some_borders_as_connection_candidates(w, h, connection_borders),
            },
        ),
    }
}