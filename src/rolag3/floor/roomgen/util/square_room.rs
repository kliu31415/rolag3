use std::{rc::Rc, cell::RefCell};

use crate::rolag3::floor::{room_object::{wall::basic_wall::{new_edge_wall, new_inner_wall}, room_object_def::{RoomObjectCollection, NewRoomObjectContext}, cosmetic::ground1::new_ground1}, floorgen::run::GenFloorRoomContext, rofiz::rofiz_state::RofizState};

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

pub fn init_basic_square_room_middle_walled(
    ctx: &mut GenFloorRoomContext,
    w: u32, 
    h: u32,
    mw: u32,
    mh: u32,
) -> (RofizState, RoomObjectCollection) {
    assert!(w % 2 == mw % 2, "expected w({}) to have some evenness as mw({})", w, mw);
    assert!(h % 2 == mh % 2, "expected h({}) to have some evenness as mh({})", h, mh);
    assert!(mw + 10 <= w, "expected mw({}) to be much less than w({})", mw, w);
    assert!(mh + 10 <= h, "expected mh({}) to be much less than h({})", mh, h);
    assert!(mw > 0);
    assert!(mh > 0);
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

    let mx1 = (w - mw) / 2;
    let mx2 = (w + mw) / 2;
    let my1 = (h - mh) / 2;
    let my2 = (h + mh) / 2;
    for x in mx1..=mx2 {
        for y in my1..=my2 {
            let wall = new_inner_wall(&mut nro_ctx, wall_theme, x, y);
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
    }

    let ground = new_ground1(&mut nro_ctx, ground_theme, 1, 1, mx1 - 1, h - 2);
    room_objects.add(Rc::new(RefCell::new(ground)));
    let ground = new_ground1(&mut nro_ctx, ground_theme, mx2 + 1, 1, w - mx2 - 2, h - 2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let ground = new_ground1(&mut nro_ctx, ground_theme, mx1, 1, mx2 - mx1 + 1, my1 - 1);
    room_objects.add(Rc::new(RefCell::new(ground)));
    let ground = new_ground1(&mut nro_ctx, ground_theme, mx1, my2 + 1, mx2 - mx1 + 1, h - my2 - 2);
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