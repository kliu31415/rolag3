use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, unit::enemy2::new_enemy2}, rofiz::rofiz_state::RofizState, room::{RoomBuilderReq, RoomBuilder}};

use super::util::connection_candidates::all_borders_as_connection_candidates;

pub fn get_gen_room_fn_test_room2() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_test_room2(ctx)
    })
}

fn make_test_room2(ctx: &mut GenFloorRoomContext) -> GenFloorRoomResponse {
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();

    let width = 30;
    let height = 30;

    for i in 0..30 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i, 29);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..29 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, 0, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, 29, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut new_floor_object_ctx, ground_theme, 1, 1, 28, 28);
    room_objects.add(Rc::new(RefCell::new(ground)));

    for i in 1..5 {
        for j in 4..7 {
            let enemy = new_enemy2(&mut new_floor_object_ctx, (15 + i*2) as f64, (15 + j*2) as f64);
            room_objects.add(Rc::new(RefCell::new(enemy)));
        }
    }

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width,
                height,
                room_objects,
                rofiz,
                ttc: 20.0,
                connection_candidates: all_borders_as_connection_candidates(width, height),
            }
        )
    }
}