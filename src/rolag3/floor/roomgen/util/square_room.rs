use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{room_object::{wall::basic_wall::new_edge_wall, room_object_def::{RoomObjectCollection, NewRoomObjectContext}, cosmetic::ground1::new_ground1}, floorgen::run::GenFloorRoomContext, rofiz::rofiz_state::RofizState};

pub fn init_basic_square_room(
    ctx: &mut GenFloorRoomContext,
    w: u32, 
    h: u32, 
) -> (RofizState, RoomObjectCollection) {
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut nro_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();
    for i in 0..w {
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, i, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, i, h - 1);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    for i in 1..h-1 {
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, 0, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, w - 1, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut nro_ctx, ground_theme, 1, 1, w - 2, h - 2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    (rofiz, room_objects)
}

pub fn init_basic_square_room_no_ground(
    ctx: &mut GenFloorRoomContext,
    w: u32, 
    h: u32, 
) -> (RofizState, RoomObjectCollection) {
    let wall_theme = ctx.wall_theme;
    let mut rofiz = RofizState::new();
    let mut nro_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();
    for i in 0..w {
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, i, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, i, h - 1);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    for i in 1..h-1 {
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, 0, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = new_edge_wall(&mut nro_ctx, wall_theme, w - 1, i);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    (rofiz, room_objects)
}