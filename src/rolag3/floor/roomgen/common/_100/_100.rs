use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, rofiz::rofiz_state::RofizState, room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection}, wall::basic_wall::new_edge_wall, cosmetic::ground1::new_ground1, tiles::key_tile::{new_key_tile, KEY_TILE_SIDE_LEN}}, room::{RoomBuilderReq, RoomBuilder}, roomgen::util::{connection_candidates::some_borders_as_connection_candidates, logfp::new_logfp_v1}};

/* Common100 contains three orbs moving around the walls, with lasers arcing between the orbs. To clear the room, the
   player must activate 4 key tiles. One key tile is in each quadrant of the room.
*/

pub fn get_gen_room_fn_common100(room_side_len: u32) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, room_side_len)
    })
}

fn make_room(ctx: &mut GenFloorRoomContext, room_side_len: u32) -> GenFloorRoomResponse {
    assert!(room_side_len >= 10, "room_side_len of {} is too small", room_side_len);
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut nro_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();

    let width = room_side_len;
    let height = room_side_len;

    for i in 0..width {
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, i, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, i, height-1);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..(height-1) {
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, 0, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, width-1, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut nro_ctx, ground_theme, 1, 1, width-2, height-2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let key_tile1_x = width / 4 - 1;
    let key_tile1_y = height / 4 - 1;
    let key_tiles = [new_key_tile(&mut nro_ctx, key_tile1_x, key_tile1_y),
        new_key_tile(&mut nro_ctx, width - key_tile1_x - KEY_TILE_SIDE_LEN, key_tile1_y),
        new_key_tile(&mut nro_ctx, width - key_tile1_x - KEY_TILE_SIDE_LEN, height - key_tile1_y - KEY_TILE_SIDE_LEN),
        new_key_tile(&mut nro_ctx, key_tile1_x, height - key_tile1_y - KEY_TILE_SIDE_LEN),
    ];
    key_tiles.into_iter().for_each(|x| room_objects.add(Rc::new(RefCell::new(x))));

    let (logfp, connection_borders) = new_logfp_v1(&mut nro_ctx, width, height);
    logfp.into_vec().into_iter().for_each(|x| room_objects.add(x));

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width,
                height,
                room_objects,
                rofiz,
                ttc: 5.0 + 0.3 * (room_side_len as f64),
                connection_candidates: some_borders_as_connection_candidates(width, height, connection_borders),
            }
        ),
    }
}