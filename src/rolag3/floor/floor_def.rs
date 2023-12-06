use std::{collections::HashMap, cell::RefCell, rc::Rc};

use super::{room::Room, room_object::{unit::player::Player, room_object_def::FloorCoordinate, tiles::room_connection::Direction}};

pub struct Floor {
    pub rooms: HashMap<RoomId, Room>,
    pub player: Rc<RefCell<Player>>,
    pub player_room_id: RoomId,
    pub floor_time: f64,
}

type RoomId = usize;

impl Floor {
    pub fn new_test1() -> Self {
        let player = Rc::new(RefCell::new(Player::new_test1()));
        let mut room = Room::new_test_room1();
        room.finalize_with_connections(vec![(0, 10, Direction::Left)]);
        player.borrow_mut().move_rooms(&mut room.rofiz);
        room.room_objects.add(player.clone());
        let mut rooms = HashMap::new();
        rooms.insert(1, room);

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