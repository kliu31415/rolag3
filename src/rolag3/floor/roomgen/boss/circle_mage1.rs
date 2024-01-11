use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, room_object::{wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, room_object_def::{NewRoomObjectContext, RoomObjectCollection}, unit::enemy::boss::circle_mage1::new_boss_circle_mage1}, rofiz::rofiz_state::RofizState, room::RoomCtorArgs};

pub fn _get_gen_room_fn_boss_circle_mage1() -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        _make_room(ctx)
    })
}

fn _make_room(ctx: &mut GenFloorRoomContext) -> GenFloorRoomResponse {
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

    let enemies = new_boss_circle_mage1(&mut new_floor_object_ctx, 25.0, 25.0);
    enemies.into_vec().into_iter().for_each(|x| room_objects.add(x));

    GenFloorRoomResponse {
        room_ctor_args: RoomCtorArgs {
            width,
            height,
            room_objects,
            rofiz,
            ttc: 50.0,
            connection_candidates: Vec::new(),
            is_hallway: false,
        }
    }
}