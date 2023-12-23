use std::{collections::HashMap, cell::RefCell, rc::Rc};

use rand::rngs::StdRng;

use crate::gfx::renderer::Renderer;

use super::{room::{Room, RoomConnectionInfo}, room_object::{unit::player::{Player, MoveRooms}, room_object_def::{FloorCoordinate, RoomObjectId}, tiles::room_connection::Direction}, rooms::{maze1::make_room_maze1, boss_room1::make_boss_room1}};

pub struct Floor {
    pub rooms: HashMap<RoomId, Room>,
    pub player: Rc<RefCell<Player>>,
    pub player_room_id: RoomId,
    pub floor_time: f64,
    // all room objects within a floor share the same ID counter. The reason is that some objects can move between rooms
    // on a floor. To ensure all objects in a room have a unique ID, they must be constructed with the same counter.
    // This has caused bugs when the player (id=1) and the first wall constructed in a room (id=1) collide.
    // TODO: in the future, make all room objects across all floors in a run share the same counter.
    pub room_object_id_counter: RoomObjectId,
}

pub type RoomId = usize;

impl Floor {
    // ids [0..100] are reserved for now
    pub const ROOM_OBJECT_ID_COUNTER_BEGIN: RoomObjectId = 100;
    pub const PLAYER_ROOM_OBJECT_ID: RoomObjectId = 1;

    pub fn new_test1(renderer: &mut dyn Renderer, rng: &mut StdRng) -> Self {
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let player = Rc::new(RefCell::new(Player::new_test1()));
        let mut room1 = Room::new_test_room1(rng, &mut room_object_id_counter);
        let connection1_info1 = RoomConnectionInfo {
            x: 29,
            y: 10,
            direction: Direction::Right,
            connects_to_room_id: 2,
            connects_to_x: 0,
            connects_to_y: 10,
        };
        room1.finalize_with_connections(renderer, vec![connection1_info1], rng, &mut room_object_id_counter);

        let connection1_info2 = RoomConnectionInfo {
            x: 0,
            y: 10,
            direction: Direction::Left,
            connects_to_room_id: 1,
            connects_to_x: 29,
            connects_to_y: 10,
        };
        let mut room2 = Room::new_test_room2(rng, &mut room_object_id_counter);
        room2.finalize_with_connections(renderer, vec![connection1_info2], rng, &mut room_object_id_counter);

        let mut room3 = make_room_maze1(rng, &mut room_object_id_counter, 30, 30);
        room3.finalize_with_connections(renderer, vec![], rng, &mut room_object_id_counter);

        let mut room4 = make_boss_room1(rng, &mut room_object_id_counter);
        room4.finalize_with_connections(renderer, vec![], rng, &mut room_object_id_counter);

        player.borrow_mut().move_rooms(&mut room4.rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
        room4.room_objects.add(player.clone());
        let mut rooms = HashMap::new();
        rooms.insert(1, room1);
        rooms.insert(2, room2);
        rooms.insert(3, room3);
        rooms.insert(4, room4);
        
        Self {
            rooms,
            player,
            player_room_id: 4,
            floor_time: 0.0,
            room_object_id_counter,
        }
    }

    pub fn get_current_room(&mut self) -> &mut Room {
        self.rooms.get_mut(&self.player_room_id).unwrap()
    }

    pub fn get_player_and_current_room(&mut self) -> (Rc<RefCell<Player>>, &mut Room) {
        (self.player.clone(), self.rooms.get_mut(&self.player_room_id).unwrap())
    }

    pub fn get_player_and_current_room_and_id_counter(&mut self) -> (Rc<RefCell<Player>>, &mut Room, &mut RoomObjectId) {
        (self.player.clone(), self.rooms.get_mut(&self.player_room_id).unwrap(), &mut self.room_object_id_counter)
    }

    pub fn get_player_center(&self) -> FloorCoordinate {
        self.player.borrow().get_center_point(&self.rooms.get(&self.player_room_id).as_ref().unwrap().rofiz)
    }
}