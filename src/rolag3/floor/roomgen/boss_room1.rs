use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, unit::boss1::new_boss1}, rofiz::rofiz_state::RofizState, room::{RoomBuilderReq, RoomBuilder}, floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}};

pub fn get_gen_room_fn_boss1() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_boss_room1(ctx)
    })
}

fn make_boss_room1(ctx: &mut GenFloorRoomContext,) -> GenFloorRoomResponse {
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();

    let width = 50;
    let height = 50;

    for i in 0..50 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i, 49);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..49 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, 0, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, 49, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut new_floor_object_ctx, ground_theme, 1, 1, 48, 48);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let enemy = new_boss1(&mut new_floor_object_ctx, 25.0, 25.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    GenFloorRoomResponse {
        room_builder: RoomBuilder::new(
            RoomBuilderReq {
                width,
                height,
                room_objects,
                rofiz,
                ttc: 50.0,
                connection_candidates: Vec::new(),
            }
        ),
    }
}