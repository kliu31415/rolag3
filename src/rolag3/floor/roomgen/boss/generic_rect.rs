use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{floorgen::run::{GenFloorRoomContext, GenFloorRoomResponse}, room_object::{wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, room_object_def::{NewRoomObjectContext, RoomObjectCollection, RoomObject}, tiles::next_floor_tile::{new_next_floor_tile, NEXT_FLOOR_TILE_SIDE_LEN}}, rofiz::rofiz_state::RofizState, room::RoomCtorArgs};

pub fn get_gen_room_fn_boss_generic_rect(
    width: u32, 
    height: u32,
    make_enemy_fn: Box<dyn Fn(&mut NewRoomObjectContext) -> Box<[Rc<RefCell<dyn RoomObject>>]>>,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room(ctx, width, height, make_enemy_fn.as_ref())
    })
}

fn make_room(
    ctx: &mut GenFloorRoomContext, 
    width: u32, 
    height: u32,
    make_enemy_fn: &dyn Fn(&mut NewRoomObjectContext) -> Box<[Rc<RefCell<dyn RoomObject>>]>,
) -> GenFloorRoomResponse {
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();

    for i in 0..width {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i, height-1);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..(height-1) {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, 0, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, width-1, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut new_floor_object_ctx, ground_theme, 1, 1, width-2, height-2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let next_floor_tile = new_next_floor_tile(&mut new_floor_object_ctx, 
        (width-NEXT_FLOOR_TILE_SIDE_LEN)/2,
        (height-NEXT_FLOOR_TILE_SIDE_LEN)/2);
    room_objects.add(Rc::new(RefCell::new(next_floor_tile)));

    let enemies = (make_enemy_fn)(&mut new_floor_object_ctx);
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