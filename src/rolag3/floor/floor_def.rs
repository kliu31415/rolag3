use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{gfx::renderer::Renderer, rolag3::floor::{roomgen::empty1::get_gen_room_fn_empty1, floorgen::run::GenFloorRoomFn, room_object::{cosmetic::ground1::GroundTheme, wall::basic_wall::WallTheme}, draw::Color}, util::rng::Rng};

use super::{room::{Room, RoomConnectionInfo}, room_object::{unit::player::{Player, MoveRooms}, room_object_def::{FloorCoordinate, RoomObjectId}, tiles::room_connection::Direction}, roomgen::{boss_room1::get_gen_room_fn_boss1, test_room1::get_gen_room_fn_test_room1, test_room2::get_gen_room_fn_test_room2, maze1::get_gen_room_fn_maze1, boss::circle_mage1::get_gen_room_fn_boss_circle_mage1}, floorgen::run::{gen_floor, GenFloorArgs, GenFloorRoomContext}};

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
    pub floor_w: u32,
    pub floor_h: u32,
}

pub type RoomId = usize;

impl Floor {
    // ids [0..100] are reserved for now
    pub const ROOM_OBJECT_ID_COUNTER_BEGIN: RoomObjectId = 100;
    pub const PLAYER_ROOM_OBJECT_ID: RoomObjectId = 1;

    pub fn new_test1(renderer: &mut dyn Renderer, rng: &mut Rng) -> Self {
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let player = Rc::new(RefCell::new(Player::new_test1()));
        let ground_theme = GroundTheme::Monocolor(Color::new(0.02, 0.0, 0.0, 1.0));
        let mut gfr_ctx = GenFloorRoomContext {
            rng,
            room_object_id_counter: &mut room_object_id_counter,
            ground_theme,
            wall_theme: WallTheme::Monocolor(Color::new(0.1, 0.2, 0.3, 1.0)),
        };
        let mut room1 = (get_gen_room_fn_test_room1())(&mut gfr_ctx).room_ctor_args.to_room();
        room1.upper_left_x = 0;
        room1.upper_left_y = 0;
        let mut room2 = (get_gen_room_fn_test_room2())(&mut gfr_ctx).room_ctor_args.to_room();
        room2.upper_left_x = 30;
        room2.upper_left_y = 0;
        let mut room3 = (get_gen_room_fn_maze1(24, 24))(&mut gfr_ctx).room_ctor_args.to_room();
        room3.upper_left_x = 30;
        room3.upper_left_y = 30;
        let mut room4 = (get_gen_room_fn_boss1())(&mut gfr_ctx).room_ctor_args.to_room();
        room4.upper_left_x = 150;
        room4.upper_left_y = 30;

        let connection1_info1 = RoomConnectionInfo {
            x: 29,
            y: 10,
            direction: Direction::Right,
            connects_to_room_id: 2,
            connects_to_x: 0,
            connects_to_y: 10,
        };

        room1.finalize_with_connections(renderer, vec![connection1_info1], rng, &mut room_object_id_counter, ground_theme);

        let connection1_info2 = RoomConnectionInfo {
            x: 0,
            y: 10,
            direction: Direction::Left,
            connects_to_room_id: 1,
            connects_to_x: 29,
            connects_to_y: 10,
        };
        room2.finalize_with_connections(renderer, vec![connection1_info2], rng, &mut room_object_id_counter, ground_theme);

        room3.finalize_with_connections(renderer, vec![], rng, &mut room_object_id_counter, ground_theme);

        room4.finalize_with_connections(renderer, vec![], rng, &mut room_object_id_counter, ground_theme);

        player.borrow_mut().move_rooms(&mut room1.rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
        room1.room_objects.add(player.clone());
        let mut rooms = HashMap::new();
        rooms.insert(1, room1);
        rooms.insert(2, room2);
        rooms.insert(3, room3);
        rooms.insert(4, room4);
        
        Self {
            rooms,
            player,
            player_room_id: 1,
            floor_time: 0.0,
            room_object_id_counter,
            floor_w: 200,
            floor_h: 200,
        }
    }

    pub fn new_test2(renderer: &mut dyn Renderer, rng: &mut Rng) -> Self {
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let player = Rc::new(RefCell::new(Player::new_test1()));
        let gen_initial_room_fn = GenFloorRoomFn {weight: 1.0, func: get_gen_room_fn_empty1(20, 20, 20, 20)};
        let gen_normal_room_fns = vec![GenFloorRoomFn {weight: 1.0, func: get_gen_room_fn_empty1(20, 50, 20, 50)}];
        let ground_theme = GroundTheme::Monocolor(Color::new(0.02, 0.0, 0.0, 1.0));
        let wall_theme = WallTheme::Monocolor(Color::new(0.1, 0.2, 0.3, 1.0));
        let gf_args = GenFloorArgs {
            grid_w: 256,
            grid_h: 256,
            ground_theme,
            wall_theme,
            ttc_min: 50.0,
            ttc_max: 53.0,
            gen_initial_room_fn,
            gen_normal_room_fns,
            rng,
            room_object_id_counter: &mut room_object_id_counter,
            save_debug_data: true,
        };
        let mut gf_result = gen_floor(gf_args);
        //gf_result.rooms.iter_mut().for_each(|room| room.finalize_with_connections(renderer, vec![], rng, &mut room_object_id_counter));
        let mut rooms = gf_result.rooms.drain(..).enumerate().collect::<HashMap<_, _>>();
        assert!(rooms.len() >= 1);
        player.borrow_mut().move_rooms(&mut rooms.get_mut(&0).unwrap().rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
        rooms.get_mut(&0).unwrap().room_objects.add(player.clone());
        gf_result.connections.drain(..).enumerate().for_each(|(rid, rci)| {
            rooms.get_mut(&rid).unwrap().finalize_with_connections(renderer, rci, rng, &mut room_object_id_counter, ground_theme);
        });

        Self {
            rooms,
            player,
            player_room_id: 0,
            floor_time: 0.0,
            room_object_id_counter,
            floor_w: gf_result.floor_w,
            floor_h: gf_result.floor_h,
        }
    }

    pub fn new_test3(renderer: &mut dyn Renderer, rng: &mut Rng) -> Self {
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let player = Rc::new(RefCell::new(Player::new_test1()));
        let ground_theme = GroundTheme::Monocolor(Color::new(0.02, 0.0, 0.0, 1.0));
        let mut gfr_ctx = GenFloorRoomContext {
            rng,
            room_object_id_counter: &mut room_object_id_counter,
            ground_theme,
            wall_theme: WallTheme::Monocolor(Color::new(0.1, 0.2, 0.3, 1.0)),
        };
        let mut room1 = (get_gen_room_fn_boss_circle_mage1())(&mut gfr_ctx).room_ctor_args.to_room();
        room1.upper_left_x = 0;
        room1.upper_left_y = 0;
        room1.finalize_with_connections(renderer, Vec::new(), rng, &mut room_object_id_counter, ground_theme);

        player.borrow_mut().move_rooms(&mut room1.rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
        room1.room_objects.add(player.clone());
        let mut rooms = HashMap::new();
        rooms.insert(1, room1);
        
        Self {
            rooms,
            player,
            player_room_id: 1,
            floor_time: 0.0,
            room_object_id_counter,
            floor_w: 200,
            floor_h: 200,
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