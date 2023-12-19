use std::{rc::Rc, cell::RefCell};

use rand::rngs::ThreadRng;

use super::{room_object::{unit::{enemy2::new_enemy2, enemy3::new_enemy3, enemy1::new_enemy1, boss1::new_boss1, enemy4::new_enemy4, enemy5::new_enemy5}, room_object_def::{NewRoomObjectContext, RoomObjectCollection, RoomObjectId}, wall::basic_wall::BasicWall, tiles::{room_connection::{RoomConnection, Direction}, black_hole::new_black_hole, accel_tile::new_accel_tile}, damage::DamageColor, cosmetic::ground1::new_ground1}, draw::Color, rofiz::rofiz_state::RofizState, floor_def::RoomId};

pub struct Room {
    pub upper_left_x: u32,
    pub upper_left_y: u32,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<RoomTile>>,
    pub room_objects: RoomObjectCollection,
    pub rofiz: RofizState,
    pub room_object_id_counter: RoomObjectId,
    pub room_time: f64,
    pub room_cleared_at_time: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
pub enum RoomTile {
    _NotInRoom,
    Ground,
    Wall,
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

        let width = 30;
        let height = 30;

        let mut tiles = vec![vec![RoomTile::Ground; height as usize]; width as usize];

        for i in 0..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            tiles[i as usize][0] = RoomTile::Wall;
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 29, Color::new(0.1, 0.2, 0.3, 1.0));
            tiles[i as usize][29] = RoomTile::Wall;
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
        
        for i in 1..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            tiles[0][i as usize] = RoomTile::Wall;
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, 29, i, Color::new(0.1, 0.2, 0.3, 1.0));
            tiles[29][i as usize] = RoomTile::Wall;
            room_objects.add(Rc::new(RefCell::new(wall)));
        }

        let ground = new_ground1(&mut new_floor_object_ctx, Color::new(0.02, 0.0, 0.0, 1.0), 1, 1, 28, 29);
        room_objects.add(Rc::new(RefCell::new(ground)));

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

        let enemy = new_enemy5(&mut new_floor_object_ctx, 16.0, 25.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let enemy = new_boss1(&mut new_floor_object_ctx, 5.0, 25.0);
        room_objects.add(Rc::new(RefCell::new(enemy)));

        let bhole = new_black_hole(&mut new_floor_object_ctx, Some(DamageColor::Green), 15.0, 15.0);
        room_objects.add(Rc::new(RefCell::new(bhole)));

        let accel_tile = new_accel_tile(&mut new_floor_object_ctx, 10, 10);
        room_objects.add(Rc::new(RefCell::new(accel_tile)));

        Self {
            upper_left_x: 0,
            upper_left_y: 0,
            width,
            height,
            tiles,
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

        let width = 30;
        let height = 30;

        let mut tiles = vec![vec![RoomTile::Ground; height as usize]; width as usize];

        for i in 0..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            tiles[i as usize][0] = RoomTile::Wall;
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 29, Color::new(0.1, 0.2, 0.3, 1.0));
            tiles[i as usize][29] = RoomTile::Wall;
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
        
        for i in 1..30 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            tiles[0][i as usize] = RoomTile::Wall;
            room_objects.add(Rc::new(RefCell::new(wall)));
            let wall = BasicWall::new(&mut new_floor_object_ctx, 29, i, Color::new(0.1, 0.2, 0.3, 1.0));
            tiles[29][i as usize] = RoomTile::Wall;
            room_objects.add(Rc::new(RefCell::new(wall)));
        }

        for i in 1..5 {
            for j in 4..7 {
                let enemy = new_enemy2(&mut new_floor_object_ctx, (15 + i*2) as f64, (15 + j*2) as f64);
                room_objects.add(Rc::new(RefCell::new(enemy)));
            }
        }

        Self {
            upper_left_x: 30,
            upper_left_y: 0,
            width,
            height,
            tiles,
            room_objects,
            rofiz,
            room_object_id_counter,
            room_time: 0.0,
            room_cleared_at_time: None,
        }
    }
}