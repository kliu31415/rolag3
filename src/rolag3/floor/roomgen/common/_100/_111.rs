use std::{collections::BTreeSet, cell::RefCell, rc::Rc};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, roomgen::util::{connection_candidates::all_borders_as_connection_candidates, square_room::init_basic_square_room}, room_object::{room_object_def::NewRoomObjectContext, unit::enemy::thinstar4::rgb_circle::new_thinstar4_group, damage::DamageColor, tiles::ice_tile::new_ice_tile}, room::{RoomBuilder, RoomBuilderReq}};

/* Common111 contains starflies and ice tiles
 */

pub fn get_gen_room_fn_common111(
    w: u32, 
    h: u32, 
    num_ice_tiles: usize,
    starfly_group_sizes: Box<[usize]>,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, w, h, num_ice_tiles, &starfly_group_sizes)
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    w: u32, 
    h: u32, 
    num_ice_tiles: usize,
    starfly_group_sizes: &[usize],
) -> GenFloorRoomResponse {
    assert!(w >= 20, "w({}) is too low", w);
    assert!(h >= 20, "h({}) is too low", h);
    let mut rng = ctx.rng.spawn_child();
    let (mut rofiz, mut room_objects) = init_basic_square_room(ctx, w, h);
    let nro_ctx = &mut NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);

    for sgs in starfly_group_sizes {
        let damage_colors = (0..*sgs).map(|_| {
            rng.sample_slice_uniform(&[DamageColor::Red, DamageColor::Green, DamageColor::Blue])
        }).collect::<Box<_>>();
        let x = rng.gen_f64_range(0.3 .. 0.7) * (w as f64);
        let y = rng.gen_f64_range(0.3 .. 0.7) * (h as f64);
        let enemies = new_thinstar4_group(nro_ctx, &damage_colors, x, y);
        enemies.into_vec().into_iter().for_each(|x| room_objects.add(x));
    }

    let mut num_iter = 0;
    let mut ice_tile_xy = BTreeSet::new();
    while ice_tile_xy.len() < num_ice_tiles {
        num_iter += 1;
        assert!(num_iter < 10000);

        let x = rng.gen_u32_range(3..(w-3));
        let y = rng.gen_u32_range(3..(h-3));
        ice_tile_xy.insert((x, y));
    }

    for (x, y) in ice_tile_xy {
        let ice_tile = new_ice_tile(nro_ctx, x, y);
        room_objects.add(Rc::new(RefCell::new(ice_tile)));
    }

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width: w,
                height: h,
                room_objects,
                rofiz,
                ttc: 0.1 * f64::sqrt((w * h) as f64) 
                     + 3.0 * starfly_group_sizes.iter().sum::<usize>() as f64
                     + 0.2 * (num_ice_tiles as f64),
                connection_candidates: all_borders_as_connection_candidates(w, h),
            },
        ),
    }
}