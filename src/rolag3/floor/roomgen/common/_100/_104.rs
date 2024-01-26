use std::{cell::RefCell, rc::Rc, ops::Range};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, tiles::{damage_tile::new_damage_tile_with_is_active_fn, accel_tile::new_accel_tile, key_tile::new_key_tile}}, room::{RoomBuilder, RoomBuilderReq}};

/* Common104 contains a large strip of X tiles in the middle in a plus formation. 
   - There are 4 key tiles, one in each corner.
   - There are 3 accel tiles in each corner that rotate around
 */

pub fn get_gen_room_fn_common104(
    one_third: Range<u32>,
    damage_tile_flash_mult: f64,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, one_third.clone(), damage_tile_flash_mult)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, one_third_range: Range<u32>, flash_mult: f64) -> GenFloorRoomResponse {
    let mut rng = ctx.rng.spawn_child();
    let one_third = rng.gen_u32_range(one_third_range.clone());
    assert!(one_third >= 10, "one_third({}) is too small. Range is {:?}", one_third, one_third_range);
    assert!(one_third <= 20, "one_third({}) is too large. Range is {:?}", one_third, one_third_range);
    assert!(flash_mult >= 0.03, "flash_mult({}) is too small", flash_mult);
    assert!(flash_mult <= 0.07, "flash_mult({}) is too large", flash_mult);
    let w = 3 * one_third + 2;
    let h = 3 * one_third + 2;
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    for i in 0..3 {
        for j in 0..3 {
            if (i==0 || i==2) && (j==0 || j==2) {
                continue;
            }
            for a in 0..one_third {
                for b in 0..one_third {
                    let x = i * one_third + a + 1;
                    let y = j * one_third + b + 1;
                    let period = flash_mult * one_third as f64;
                    let is_active = Box::new(move |age| age % (2.0 * period) < period);
                    let enemy = new_damage_tile_with_is_active_fn(nro_ctx, x, y, is_active);
                    room_objects.add(Rc::new(RefCell::new(enemy)));
                }
            }
        }
    }

    for i in 0..2 {
        for j in 0..2 {
            for u in 0..2 {
                for v in 0..2 {
                    let a = 4;
                    let b = 5;
                    let x = one_third - a - b*u + i * (one_third + 2*a + 2*b*u);
                    let y = one_third - a - b*v + j * (one_third + 2*a + 2*b*v);
                    if u == 1 && v == 1 {
                        let key_tile = new_key_tile(nro_ctx, x, y);
                        room_objects.add(Rc::new(RefCell::new(key_tile)));
                    } else {
                        let a = rng.gen_f64_range(1.2 .. 1.7);
                        let b = rng.gen_f64_range(0.0 .. 2.0*std::f64::consts::PI);
                        let accel_tile = new_accel_tile(nro_ctx, x, y, Box::new(move |t| a * t + b));
                        room_objects.add(Rc::new(RefCell::new(accel_tile)));
                    }
                }
            }
        }
    }

    let connection_candidates = all_borders_as_connection_candidates(w as u32, h as u32)
        .into_iter()
        .filter(|(x, y, _)| (*x + 1 < one_third || *x > 2 * one_third + 1) && (*y + 1 < one_third || *y > 2 * one_third + 1))
        .collect();

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 2.0 * one_third as f64,
                connection_candidates,
            },
        ),
    }
}