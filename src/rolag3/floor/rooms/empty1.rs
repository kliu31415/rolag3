use std::{rc::Rc, cell::RefCell, ops::Range};

use rand::{rngs::StdRng, Rng};

use crate::rolag3::floor::{room_object::{room_object_def::{RoomObjectId, NewRoomObjectContext, RoomObjectCollection}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1}, room::Room, rofiz::rofiz_state::RofizState, draw::Color, floorgen::run::GenFloorRoomContext};

pub fn get_gen_room_fn_empty1(w_min: u32, w_max: u32, h_min: u32, h_max: u32) -> Box<dyn Fn(&mut GenFloorRoomContext) -> Room> {
    assert!(w_min <= w_max);
    assert!(h_min <= h_max);
    Box::new(move |ctx: &mut GenFloorRoomContext| {
        make_room_empty1(ctx.rng, ctx.room_object_id_counter, w_min..w_max+1, h_min..h_max+1)
    })
}

pub fn make_room_empty1(rng: &mut StdRng, room_object_id_counter: &mut RoomObjectId, widths: Range<u32>, heights: Range<u32>) -> Room {
    let width = rng.gen_range(widths);
    let height = rng.gen_range(heights);
    let ttc = 0.1 * f64::sqrt((width * height) as f64);

    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, room_object_id_counter, 0.0, rng);
    let mut room_objects = RoomObjectCollection::new();

    for x in 0..width {
        let wall = BasicWall::new(&mut new_floor_object_ctx, x, 0, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));

        let wall = BasicWall::new(&mut new_floor_object_ctx, x, height-1, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    for y in 1..height-1 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, 0, y, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));

        let wall = BasicWall::new(&mut new_floor_object_ctx, width-1, y, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut new_floor_object_ctx, Color::new(0.02, 0.0, 0.0, 1.0), 1, 1, width-2, height-2);
    room_objects.add(Rc::new(RefCell::new(ground)));

    Room {
        upper_left_x: 0,
        upper_left_y: 0,
        width,
        height,
        tiles: Vec::new(),
        room_objects,
        rofiz,
        room_time: 0.0,
        room_cleared_at_time: None,
        minimap_texture: None,
        ttc,
    }
}