use std::{collections::HashMap, cell::RefCell, rc::Rc};

use rand::rngs::ThreadRng;

use crate::gfx::renderer::Renderer;

use super::{room::{Room, RoomConnectionInfo}, room_object::{unit::player::{Player, MoveRooms}, room_object_def::FloorCoordinate, tiles::room_connection::Direction}};

pub struct Floor {
    pub rooms: HashMap<RoomId, Room>,
    pub player: Rc<RefCell<Player>>,
    pub player_room_id: RoomId,
    pub floor_time: f64,
}

pub type RoomId = usize;

impl Floor {
    pub fn new_test1(renderer: &mut dyn Renderer, rng: &mut ThreadRng) -> Self {
        let player = Rc::new(RefCell::new(Player::new_test1()));
        let mut room1 = Room::new_test_room1(rng);
        let connection1_info1 = RoomConnectionInfo {
            x: 29,
            y: 10,
            direction: Direction::Right,
            connects_to_room_id: 2,
            connects_to_x: 0,
            connects_to_y: 10,
        };
        room1.finalize_with_connections(renderer, vec![connection1_info1], rng);
        player.borrow_mut().move_rooms(&mut room1.rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
        room1.room_objects.add(player.clone());

        let connection1_info2 = RoomConnectionInfo {
            x: 0,
            y: 10,
            direction: Direction::Left,
            connects_to_room_id: 1,
            connects_to_x: 29,
            connects_to_y: 10,
        };
        let mut room2 = Room::new_test_room2(rng);
        room2.finalize_with_connections(renderer, vec![connection1_info2], rng);

        let mut rooms = HashMap::new();
        rooms.insert(1, room1);
        rooms.insert(2, room2);

        Self {
            rooms,
            player,
            player_room_id: 1,
            floor_time: 0.0,
        }
    }

    pub fn get_current_room(&mut self) -> &mut Room {
        self.rooms.get_mut(&self.player_room_id).unwrap()
    }

    pub fn get_player_and_current_room(&mut self) -> (Rc<RefCell<Player>>, &mut Room) {
        (self.player.clone(), self.rooms.get_mut(&self.player_room_id).unwrap())
    }

    pub fn get_player_center(&self) -> FloorCoordinate {
        self.player.borrow().get_center_point(&self.rooms.get(&self.player_room_id).as_ref().unwrap().rofiz)
    }
}