use std::{rc::Rc, cell::RefCell};

use super::{room_object::{unit::player::Player, room_object_def::{NewRoomObjectContext, RoomObjectCollection, RoomObjectId}, wall::basic_wall::BasicWall}, draw::Color, rofiz::rofiz_state::RofizState};

pub struct Room {
    pub player: Rc<RefCell<Player>>,
    pub room_objects: RoomObjectCollection,
    pub rofiz: RofizState,
    pub room_object_id_counter: RoomObjectId,
}

impl Room {
    pub fn new_test_room1() -> Self {
        let mut rofiz = RofizState::new();
        let mut room_object_id_counter = 0;
        let mut new_floor_object_ctx = NewRoomObjectContext::new(&mut rofiz, &mut room_object_id_counter);
        let mut room_objects = RoomObjectCollection::new();

        for i in 0..20 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, i, 0, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
        
        for i in 1..20 {
            let wall = BasicWall::new(&mut new_floor_object_ctx, 0, i, Color::new(0.1, 0.2, 0.3, 1.0));
            room_objects.add(Rc::new(RefCell::new(wall)));
        }
        let player = Rc::new(RefCell::new(Player::new_test1(&mut new_floor_object_ctx)));
        room_objects.add(player.clone());

        rofiz.finalize_start_floor();

        Self {
            player,
            room_objects,
            rofiz,
            room_object_id_counter,
        }
    }
}