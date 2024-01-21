use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{room_object::{wall::basic_wall::BasicWall, room_object_def::{RoomObjectCollection, NewRoomObjectContext}, cosmetic::ground1::new_ground1}, floorgen::run::GenFloorRoomContext, rofiz::rofiz_state::RofizState};

pub fn init_basic_square_room(
    ctx: &mut GenFloorRoomContext,
    w: usize, 
    h: usize, 
) -> (RofizState, RoomObjectCollection) {
    let wall_theme = ctx.wall_theme;
    let ground_theme = ctx.ground_theme;
    let mut rofiz = RofizState::new();
    let mut nro_ctx = NewRoomObjectContext::from_gfr_ctx(&mut rofiz, ctx);
    let mut room_objects = RoomObjectCollection::new();
    for i in 0..w {
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, i as u32, 0);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, i as u32, h as u32 - 1);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    for i in 1..h-1 {
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, 0, i as u32);
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut nro_ctx, wall_theme, (w - 1) as u32, i as u32);
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut nro_ctx, ground_theme, 1, 1, w as u32 - 2, h as u32 - 2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    (rofiz, room_objects)
}