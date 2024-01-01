use std::{rc::Rc, cell::RefCell};

use rand::rngs::StdRng;

use crate::rolag3::floor::{room_object::{room_object_def::{NewRoomObjectContext, RoomObjectCollection, RoomObjectId}, wall::basic_wall::BasicWall, cosmetic::ground1::new_ground1, unit::boss1::new_boss1}, rofiz::rofiz_state::RofizState, room::Room, draw::Color};

pub fn make_boss_room1(rng: &mut StdRng, room_object_id_counter: &mut RoomObjectId) -> Room {
    let mut rofiz = RofizState::new();
    let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, room_object_id_counter, 0.0, rng);
    let mut room_objects = RoomObjectCollection::new();

    let width = 50;
    let height = 50;

    for i in 0..50 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, i, 49, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
    }
    
    for i in 1..49 {
        let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
        let wall = BasicWall::new(&mut new_floor_object_ctx, 49, i, Color::new(0.1, 0.2, 0.3, 1.0));
        room_objects.add(Rc::new(RefCell::new(wall)));
    }

    let ground = new_ground1(&mut new_floor_object_ctx, Color::new(0.02, 0.0, 0.0, 1.0), 1, 1, 48, 48);
    room_objects.add(Rc::new(RefCell::new(ground)));

    let enemy = new_boss1(&mut new_floor_object_ctx, 25.0, 25.0);
    room_objects.add(Rc::new(RefCell::new(enemy)));

    Room {
        upper_left_x: 60,
        upper_left_y: 0,
        width,
        height,
        tiles: Vec::new(),
        room_objects,
        rofiz,
        room_time: 0.0,
        room_cleared_at_time: None,
        minimap_texture: None,
        ttc: 50.0,
        connection_candidates: Vec::new(),
    }
}