use std::{rc::Rc, cell::RefCell};

use rand::rngs::ThreadRng;

use super::{room_object::{unit::{enemy2::new_enemy2, enemy3::new_enemy3, enemy1::new_enemy1, boss1::new_boss1, enemy4::new_enemy4}, room_object_def::{NewRoomObjectContext, RoomObjectCollection, RoomObjectId}, wall::basic_wall::BasicWall, tiles::room_connection::{RoomConnection, Direction}}, draw::Color, rofiz::rofiz_state::RofizState, floor_def::RoomId};

pub struct Room {
    pub room_objects: RoomObjectCollection,
    pub rofiz: RofizState,
    pub room_object_id_counter: RoomObjectId,
    pub room_time: f64,
    pub room_cleared_at_time: Option<f64>,
}

#[derive(Debug, Copy, Clone)]
pub struct RoomConnectionInfo {
    pub x: u32,
    pub y: u32,
    pub direction: Direction,
    pub connects_to_room_id: RoomId,
    pub connects_to_x: u32,
    pub connects_to_y: u32,
}

impl Room {
    // ids [0..100] are reserved for now
    const ROOM_OBJECT_ID_COUNTER_BEGIN: RoomObjectId = 100;
    pub const PLAYER_ROOM_OBJECT_ID: RoomObjectId = 1;

    pub fn finalize_with_connections(&mut self, connections: Vec<RoomConnectionInfo>, rng: &mut ThreadRng) {
        for c in connections.iter() {
            for (x, y) in RoomConnection::get_occupied_coords(c.x, c.y, c.direction) {
                self.room_objects.remove_wall_at(x, y);
                // we have to explicitly remove the basic wall from Rofiz. Rofiz has a built-in assert when running
                // that ensures all Rofiz objects corresponding to basic walls have Rc > 1, because basic walls 
                // can never be deleted after the floor starts. If we don't explicitly remove the wall, it'll remain
                // in Rofiz with Rc=1, which causes a panic.
                self.rofiz.remove_wall_at(x, y);
            }
        }

        let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut self.rofiz, &mut self.room_object_id_counter, 0.0, rng);
        for c in connections {
            let connection = RoomConnection::new(&mut new_floor_object_ctx, c);
            self.room_objects.add(Rc::new(RefCell::new(connection)));
        }
        self.rofiz.finalize_start_floor();
    }

    pub fn new_test_room1(rng: &mut ThreadRng) -> Self {
        let mut rofiz = RofizState::new();
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, &mut room_object_id_counter, 0.0, rng);
        let mut room_objects = RoomObjectCollection::new();

        for i in 0..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 30, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
        
        for i in 1..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, 29, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }

        for i in 1..5 {
            for j in 1..4 {
                let enemy = new_enemy1(&mut new_floor_object_ctx, (6 + i*2) as f64, (6 + j*2) as f64);
                room_objects.add(Rc::new(RefCell::new(enemy)));
            }
            for j in 4..7 {
                let enemy = new_enemy2(&mut new_floor_object_ctx, (6 + i*2) as f64, (6 + j*2) as f64);
                room_objects.add(Rc::new(RefCell::new(enemy)));
            }
        }

        let enemy = new_enemy3(&mut new_floor_object_ctx, 9.0, 25.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_enemy4(&mut new_floor_object_ctx, 12.0, 25.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));


        let enemy = new_boss1(&mut new_floor_object_ctx, 5.0, 25.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        Self {
            room_objects,
            rofiz,
            room_object_id_counter,
            room_time: 0.0,
            room_cleared_at_time: None,
        }
    }

    pub fn new_test_room2(rng: &mut ThreadRng) -> Self {
        let mut rofiz = RofizState::new();
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, &mut room_object_id_counter, 0.0, rng);
        let mut room_objects = RoomObjectCollection::new();

        for i in 0..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 30, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
        
        for i in 1..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, 29, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }

        for i in 1..5 {
            for j in 4..7 {
                let enemy = new_enemy2(&mut new_floor_object_ctx, (15 + i*2) as f64, (15 + j*2) as f64);
                room_objects.add(Rc::new(RefCell::new(enemy)));
            }
        }

        Self {
            room_objects,
            rofiz,
            room_object_id_counter,
            room_time: 0.0,
            room_cleared_at_time: None,
        }
    }
}