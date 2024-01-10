use std::{cell::RefCell, rc::Rc};

use crate::rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, tiles::damage_tile::new_damage_tile}, rofiz::rofiz_state::RofizState, room::RoomCtorArgs, floorgen::run::{GenFloorRoomResponse, GenFloorRoomContext}};

use super::util::rectangular_maze::make_rectangular_maze;

pub fn get_gen_room_fn_maze1(
    w: usize, 
    h: usize,
) -> Box<dyn Fn(&mut GenFloorRoomContext) -> GenFloorRoomResponse> {
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room_maze1(ctx, w, h)
    })
}

fn make_room_maze1(ctx: &mut GenFloorRoomContext, maze_w: usize, maze_h: usize) -> GenFloorRoomResponse {
    let maze = make_rectangular_maze(ctx.rng, maze_w, maze_h, 10);
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();

    let room_w = 5 * maze_w + 1;
    let room_h = 5 * maze_h + 1;

    for i in 0..room_w {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i as u32, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, i as u32, room_h as u32 - 1);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..(room_h-1) {
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, 0, i as u32);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, wall_theme, (room_w - 1) as u32, i as u32);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut new_floor_object_ctx, ground_theme, 1, 1, room_w as u32 - 2, room_h as u32 - 2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let maze_wall_array = maze.get_maze_wall_array(4);
    for x in 0..(maze_wall_array.len()) {
        for y in 0..(maze_wall_array[0].len()) {
            if maze_wall_array[x][y] {
                let tile = new_damage_tile(&mut new_floor_object_ctx,(x + 1) as u32, (y + 1) as u32);
                room_objects.add(Rc::new(RefCell::new(tile)));
            }
        }
    }

    GenFloorRoomResponse {
        room_ctor_args: RoomCtorArgs {
            width: room_w as u32,
            height: room_h as u32,
            room_objects,
            rofiz,
            ttc: 20.0,
            connection_candidates: Vec::new(),
            is_hallway: false,
        },
    }
}