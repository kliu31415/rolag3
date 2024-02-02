use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{gfx::renderer::Renderer, rolag3::floor::{roomgen::{empty1::get_gen_room_fn_empty1, common::_100::{_100::get_gen_room_fn_common100, _101::get_gen_room_fn_common101, _102::get_gen_room_fn_common102a, _103::get_gen_room_fn_common103, _104::get_gen_room_fn_common104, _105::get_gen_room_fn_common105, _106::get_gen_room_fn_common106, _107::{get_gen_room_fn_common107a, get_gen_room_fn_common107b}, _108::get_gen_room_fn_common108, _109::get_gen_room_fn_common109, _110::{get_gen_room_fn_common110a, get_gen_room_fn_common110b}, _111::get_gen_room_fn_common111, _112::get_gen_room_fn_common112, _113::{get_gen_room_fn_common113a, get_gen_room_fn_common113b}}}, floorgen::run::{GenFloorRoomFn, GenFloorRoomReqInfo}, room_object::{cosmetic::ground1::GroundTheme, wall::basic_wall::WallTheme, unit::enemy::{thinstar4::rgb_circle::new_thinstar4_group, square::rgb_star4or8::{new_square_rgb_star4, new_square_rgb_star8}, boss::{chromatic_wheel::new_boss_chromatic_wheel, mystic_prism::new_boss_mystic_prism, prismatic_prism::new_boss_prismatic_prism}, hexagon::rgb2_circle::new_hexagon_rgb2_circle, small_square::rgb_circle_or_tri::new_small_square_rgb_circle, small_square::rgb_circle_or_tri::new_small_square_rgb_tri}, damage::DamageColor}, draw::Color, rofiz::rofiz_state::RofizState}, util::rng::Prng};

use super::{room::{Room, RoomConnectionInfo}, room_object::{unit::{player::{Player, MoveRooms}, enemy::boss::{star_king::new_boss_star_king, circle_mage1::new_boss_circle_mage1, star_soldier::new_boss_star_soldier}}, room_object_def::{FloorCoordinate, RoomObjectId, RoomObjectRef, RoomObjectType, NewRoomObjectContext, RoomObject}, tiles::room_connection::Direction, sound::RoomObjSoundIdT}, roomgen::{boss_room1::get_gen_room_fn_boss1, test_room1::get_gen_room_fn_test_room1, test_room2::get_gen_room_fn_test_room2, maze1::get_gen_room_fn_maze1, boss::generic_rect::get_gen_room_fn_boss_generic_rect}, floorgen::run::{gen_floor, GenFloorArgs, GenFloorRoomContext}};

pub struct Floor {
    pub rooms: HashMap<RoomId, Room>,
    pub player: Rc<RefCell<Player>>,
    pub player_room_id: RoomId,
    pub floor_time_left: f64,
    // all room objects within a floor share the same ID counter. The reason is that some objects can move between rooms
    // on a floor. To ensure all objects in a room have a unique ID, they must be constructed with the same counter.
    // This has caused bugs when the player (id=1) and the first wall constructed in a room (id=1) collide.
    // TODO: in the future, make all room objects across all floors in a run share the same counter.
    pub room_object_id_counter: RoomObjectId,
    pub play_sound_id_counter: RoomObjSoundIdT,
    pub floor_w: u32,
    pub floor_h: u32,
}

pub type RoomId = usize;

impl Floor {
    // ids [0..100] are reserved for now
    pub const ROOM_OBJECT_ID_COUNTER_BEGIN: RoomObjectId = 100;
    pub const PLAYER_ROOM_OBJECT_REF: RoomObjectRef = RoomObjectRef {id: 1, typ: RoomObjectType::Unit };

    pub fn new_test1(renderer: &mut dyn Renderer, rng: &mut Prng, player: Rc<RefCell<Player>>) -> Self {
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let ground_theme = GroundTheme::Monocolor(Color::new(0.02, 0.0, 0.0, 1.0));
        let mut gfr_ctx = GenFloorRoomContext {
            rng,
            room_object_id_counter: &mut room_object_id_counter,
            ground_theme,
            wall_theme: WallTheme::Monocolor(Color::new(0.1, 0.2, 0.3, 1.0)),
        };
        let mut room1 = (get_gen_room_fn_test_room1())(&mut gfr_ctx).room_builder.build();
        room1.upper_left_x = 0;
        room1.upper_left_y = 0;
        let mut room2 = (get_gen_room_fn_test_room2())(&mut gfr_ctx).room_builder.build();
        room2.upper_left_x = 30;
        room2.upper_left_y = 0;
        let mut room3 = (get_gen_room_fn_maze1(24, 24))(&mut gfr_ctx).room_builder.build();
        room3.upper_left_x = 30;
        room3.upper_left_y = 30;
        let mut room4 = (get_gen_room_fn_boss1())(&mut gfr_ctx).room_builder.build();
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

        player.borrow_mut().enter_room(&mut room1.rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
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
            floor_time_left: 600.0,
            room_object_id_counter,
            play_sound_id_counter: 0,
            floor_w: 200,
            floor_h: 200,
        }
    }

    pub fn new_test2(renderer: &mut dyn Renderer, rng: &mut Prng, player: Rc<RefCell<Player>>) -> Self {
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let gen_initial_room_fn = GenFloorRoomFn {weight: 1.0, func: get_gen_room_fn_empty1(20, 20, 20, 20)};
        let gen_normal_room_fns = vec![GenFloorRoomFn {weight: 1.0, func: get_gen_room_fn_empty1(20, 50, 20, 50)}];
        let gen_req_room_info = vec![
            GenFloorRoomReqInfo { 
                num_req: 1..2, 
                funcs: vec![GenFloorRoomFn {
                    weight: 0.0, 
                    func: Box::new(|ctx: &mut GenFloorRoomContext| (get_gen_room_fn_boss_generic_rect(50, 50, true,
                        Box::new(|ctx: &mut NewRoomObjectContext| {
                            let colors = (0..25).map(|_| {
                                let randv = ctx.get_randf64();
                                if randv < 1.0 / 3.0 {
                                    DamageColor::Red
                                } else if randv < 2.0 / 3.0 {
                                    DamageColor::Green
                                } else {
                                    DamageColor::Blue
                                }
                            }).collect::<Box<_>>();
                            new_thinstar4_group(ctx, &colors, 25.0, 25.0)
                        }))(ctx))),
                }, GenFloorRoomFn {
                    weight: 0.0, 
                    func: Box::new(|ctx: &mut GenFloorRoomContext| (get_gen_room_fn_boss_generic_rect(50, 50, true,
                        Box::new(|ctx: &mut NewRoomObjectContext| {
                            Box::new([
                                Rc::new(RefCell::new(new_square_rgb_star4(ctx, DamageColor::Red, 20.0, 25.0))),
                                Rc::new(RefCell::new(new_square_rgb_star4(ctx, DamageColor::Green, 25.0, 25.0))),
                                Rc::new(RefCell::new(new_square_rgb_star4(ctx, DamageColor::Blue, 30.0, 25.0))),
                                Rc::new(RefCell::new(new_square_rgb_star8(ctx, DamageColor::Red, 20.0, 30.0))),
                                Rc::new(RefCell::new(new_square_rgb_star8(ctx, DamageColor::Green, 25.0, 30.0))),
                                Rc::new(RefCell::new(new_square_rgb_star8(ctx, DamageColor::Blue, 30.0, 30.0))),
                            ])
                        }))(ctx))),
                }, GenFloorRoomFn {
                    weight: 1.0, 
                    func: Box::new(|ctx: &mut GenFloorRoomContext| (get_gen_room_fn_boss_generic_rect(50, 50, false,
                        Box::new(|ctx: &mut NewRoomObjectContext| {
                             Box::new([Rc::new(RefCell::new(new_small_square_rgb_tri(ctx, DamageColor::Red, 25.0, 25.0))),
                                Rc::new(RefCell::new(new_small_square_rgb_circle(ctx, DamageColor::Blue, 20.0, 25.0)))])
                        }))(ctx))),
                }, GenFloorRoomFn {
                    weight: 0.0, 
                    func: Box::new(|ctx: &mut GenFloorRoomContext| (get_gen_room_fn_boss_generic_rect(50, 50, false,
                        Box::new(|ctx: &mut NewRoomObjectContext| {
                             Box::new([Rc::new(RefCell::new(new_hexagon_rgb2_circle(ctx, DamageColor::Red, 25.0, 25.0)))])
                        }))(ctx))),
                },]
            },
        ];
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
            gen_req_room_info,
            rng,
            room_object_id_counter: &mut room_object_id_counter,
            save_debug_data: true,
        };
        let mut gf_result = gen_floor(gf_args);
        //gf_result.rooms.iter_mut().for_each(|room| room.finalize_with_connections(renderer, vec![], rng, &mut room_object_id_counter));
        let mut rooms = gf_result.rooms.drain(..).enumerate().collect::<HashMap<_, _>>();
        assert!(rooms.len() >= 1);
        player.borrow_mut().enter_room(&mut rooms.get_mut(&0).unwrap().rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
        rooms.get_mut(&0).unwrap().room_objects.add(player.clone());
        gf_result.connections.drain(..).enumerate().for_each(|(rid, rci)| {
            rooms.get_mut(&rid).unwrap().finalize_with_connections(renderer, rci, rng, &mut room_object_id_counter, ground_theme);
        });

        Self {
            rooms,
            player,
            player_room_id: 0,
            floor_time_left: 600.0,
            room_object_id_counter,
            play_sound_id_counter: 0,
            floor_w: gf_result.floor_w,
            floor_h: gf_result.floor_h,
        }
    }

    pub fn new_test3(renderer: &mut dyn Renderer, rng: &mut Prng, player: Rc<RefCell<Player>>) -> Self {
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let ground_theme = GroundTheme::Monocolor(Color::new(0.02, 0.0, 0.0, 1.0));
        let mut gfr_ctx = GenFloorRoomContext {
            rng,
            room_object_id_counter: &mut room_object_id_counter,
            ground_theme,
            wall_theme: WallTheme::Monocolor(Color::new(0.1, 0.2, 0.3, 1.0)),
        };
        let make_boss_fn = if true {
            Box::new(|ctx: &mut NewRoomObjectContext| 
                vec![Rc::new(RefCell::new(new_boss_star_soldier(ctx, 25.0, 25.0))) 
                     as Rc<RefCell<dyn RoomObject>>].into_boxed_slice())
        } else {
            // prevent cargo linter from warning about unused functions
            let rofiz = &mut RofizState::new();
            let ctx = &mut NewRoomObjectContext::from_gfr_ctx(rofiz, &mut gfr_ctx);
            new_boss_circle_mage1(ctx, 0.0, 0.0);
            new_boss_star_king(ctx, 0.0, 0.0);
            new_boss_star_soldier(ctx, 0.0, 0.0);
            new_boss_chromatic_wheel(ctx, 0.0, 0.0);
            new_boss_mystic_prism(ctx, 0.0, 0.0);
            new_boss_prismatic_prism(ctx, 0.0, 0.0);
            panic!()
        };
        let mut room1 = (get_gen_room_fn_boss_generic_rect(50, 50, false, make_boss_fn))(&mut gfr_ctx).room_builder.build();
        room1.upper_left_x = 0;
        room1.upper_left_y = 0;
        room1.finalize_with_connections(renderer, Vec::new(), rng, &mut room_object_id_counter, ground_theme);

        player.borrow_mut().enter_room(&mut room1.rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
        room1.room_objects.add(player.clone());
        let mut rooms = HashMap::new();
        rooms.insert(1, room1);
        
        Self {
            rooms,
            player,
            player_room_id: 1,
            floor_time_left: 600.0,
            room_object_id_counter,
            play_sound_id_counter: 0,
            floor_w: 200,
            floor_h: 200,
        }
    }

    pub fn new_test4(renderer: &mut dyn Renderer, rng: &mut Prng, player: Rc<RefCell<Player>>) -> Self {
        let mut room_object_id_counter = Self::ROOM_OBJECT_ID_COUNTER_BEGIN;
        let gen_initial_room_fn = GenFloorRoomFn {weight: 1.0, func: get_gen_room_fn_empty1(20, 20, 20, 20)};
        let gen_normal_room_fns = vec![
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_empty1(20, 50, 20, 50)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common100(30)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common101(4, 16, 16, 5, 20)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common102a()},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common103()},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common104(12..16, 0.05)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common105()},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common106(30, 30, 10)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common107a()},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common107b()},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common108()},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common109(30, 30, 0.3)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common110a(40, 40)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common110b(40, 40)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common111(30, 30, 20, Box::new([3, 3, 3]))},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common112(30, 30)},
            GenFloorRoomFn {weight: 0.0, func: get_gen_room_fn_common113a(40, 40, 3)},
            GenFloorRoomFn {weight: 1.0, func: get_gen_room_fn_common113b(45, 45, 3)},
        ];
        let ground_theme = GroundTheme::Monocolor(Color::new(0.02, 0.0, 0.0, 1.0));
        let wall_theme = WallTheme::Monocolor(Color::new(0.1, 0.2, 0.3, 1.0));
        let gf_args = GenFloorArgs {
            grid_w: 400,
            grid_h: 400,
            ground_theme,
            wall_theme,
            ttc_min: 200.0,
            ttc_max: 250.0,
            gen_initial_room_fn,
            gen_normal_room_fns,
            gen_req_room_info: Vec::new(),
            rng,
            room_object_id_counter: &mut room_object_id_counter,
            save_debug_data: true,
        };
        let mut gf_result = gen_floor(gf_args);
        //gf_result.rooms.iter_mut().for_each(|room| room.finalize_with_connections(renderer, vec![], rng, &mut room_object_id_counter));
        let mut rooms = gf_result.rooms.drain(..).enumerate().collect::<HashMap<_, _>>();
        assert!(rooms.len() >= 1);
        player.borrow_mut().enter_room(&mut rooms.get_mut(&0).unwrap().rofiz, MoveRooms::Teleport { x: 3.0, y: 3.0 });
        rooms.get_mut(&0).unwrap().room_objects.add(player.clone());
        gf_result.connections.drain(..).enumerate().for_each(|(rid, rci)| {
            rooms.get_mut(&rid).unwrap().finalize_with_connections(renderer, rci, rng, &mut room_object_id_counter, ground_theme);
        });

        Self {
            rooms,
            player,
            player_room_id: 0,
            floor_time_left: 600.0,
            room_object_id_counter,
            play_sound_id_counter: 0,
            floor_w: gf_result.floor_w,
            floor_h: gf_result.floor_h,
        }
    }

    pub fn get_current_room(&mut self) -> &mut Room {
        self.rooms.get_mut(&self.player_room_id).unwrap()
    }

    pub fn get_player_and_current_room(&mut self) -> (Rc<RefCell<Player>>, &mut Room) {
        (self.player.clone(), self.rooms.get_mut(&self.player_room_id).unwrap())
    }

    pub fn get_player_and_current_room_and_id_counters(
        &mut self,
    ) -> (Rc<RefCell<Player>>, &mut Room, &mut RoomObjectId, &mut RoomObjSoundIdT) {
        (self.player.clone(), 
        self.rooms.get_mut(&self.player_room_id).unwrap(), 
        &mut self.room_object_id_counter,
        &mut self.play_sound_id_counter)
    }

    pub fn get_player_center(&self) -> FloorCoordinate {
        self.player.borrow().get_center_point(&self.rooms.get(&self.player_room_id).as_ref().unwrap().rofiz)
    }
}